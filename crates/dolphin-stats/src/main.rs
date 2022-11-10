//! Market stats scraper using the Dolphin API

#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{borrow::Borrow, sync::Arc};

use holaplex_indexer_dolphin_stats::{
    collections_endpoint, get_datapoint_timestamp, market_stats_endpoint, Collection,
    CollectionsResponse, Datapoint,
};
use indexer_core::{
    self,
    chrono::{DateTime, Duration},
    clap,
    clap::Parser,
    db,
    db::{
        insert_into,
        models::{DolphinStats, DolphinStats1D},
        tables::dolphin_stats,
        update, Pool,
    },
    hash::HashSet,
    prelude::*,
    url::Url,
    util,
};
use serde_json::Number;
use tokio::sync::Semaphore;

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
struct Opts {
    /// Dolphin API key
    #[arg(long, env)]
    dolphin_key: String,

    /// Maximum number of concurrent requests
    #[arg(short, long, env, default_value_t = 192)]
    jobs: usize,

    /// Request 60 days of data to compute all statistics
    ///
    /// By default only two days are requested to update the day-over-day values
    #[arg(short, long, env)]
    full: bool,

    #[command(flatten)]
    db: db::ConnectArgs,
}

#[derive(Debug, Clone, Copy)]
struct Stats<T> {
    curr_1d: T,
    curr_7d: T,
    curr_30d: T,
    last_1d: T,
    last_7d: T,
    last_30d: T,
}

// Panics if your NFT price activity occurred before Jan 1 1970.  lol.
#[inline]
#[must_use]
fn get_split(now: DateTime<Utc>, offset: Duration) -> u64 {
    (now - offset).timestamp_millis().try_into().unwrap()
}

#[inline]
#[must_use]
fn slice_stats(stats: &[Datapoint], start: u64, mid: u64) -> (&[Datapoint], &[Datapoint]) {
    let start_i = stats.partition_point(|(t, _)| *t < start);
    let mid_i = stats.partition_point(|(t, _)| *t < mid);

    debug_assert!(start_i <= mid_i);

    stats[start_i..].split_at(mid_i - start_i)
}

fn split_stats<T, E: Into<Error>>(
    inf: &Stats<u64>,
    stats: impl AsRef<[Datapoint]>,
    then: impl Fn(&[Datapoint]) -> Result<T, E>,
) -> Result<Stats<T>> {
    let stats = stats.as_ref();

    let (last_1d, curr_1d) = slice_stats(stats, inf.last_1d, inf.curr_1d);
    let (last_7d, curr_7d) = slice_stats(stats, inf.last_7d, inf.curr_7d);
    let (last_30d, curr_30d) = slice_stats(stats, inf.last_30d, inf.curr_30d);

    Ok(Stats {
        curr_1d: then(curr_1d).map_err(Into::into)?,
        curr_7d: then(curr_7d).map_err(Into::into)?,
        curr_30d: then(curr_30d).map_err(Into::into)?,
        last_1d: then(last_1d).map_err(Into::into)?,
        last_7d: then(last_7d).map_err(Into::into)?,
        last_30d: then(last_30d).map_err(Into::into)?,
    })
}

#[inline]
#[must_use]
fn is_int(n: &Number) -> bool {
    n.is_i64() || n.is_u64()
}

#[inline]
fn int_error<M: FnOnce() -> D, D: std::fmt::Display>(num: &Number, msg: M) -> Result<()> {
    Err(anyhow!("Non-integer {:?} found in {}", num, msg()))
}

#[inline]
fn check_int<M: FnOnce() -> D, D: std::fmt::Display>(num: &Number, msg: M) -> Result<()> {
    if is_int(num) {
        return Ok(());
    }

    int_error(num, msg)
}

fn check_stats<N: IntoIterator>(
    nums: N,
    msg: impl std::fmt::Display,
    sym: impl std::fmt::Debug,
) -> Result<()>
where
    N::Item: Borrow<(u64, Number)>,
{
    use std::fmt::Write;

    let mut first_err = None;
    let mut first_dup = None;
    let mut last_err = None;
    let mut last_ts = None;
    let mut err_count = 0_u64;
    let mut dup_count = 0_u64;
    for pair in nums {
        let (ts, num) = pair.borrow();

        assert!(
            !last_ts.as_ref().map_or(false, |l| l > ts),
            "Stats array for {msg} of {sym:?} was not sorted!"
        );

        if last_ts.as_ref().map_or(false, |l| l == ts) {
            if first_dup.is_some() {
                dup_count += 1;
            } else {
                first_dup = Some(*ts);
            }
        }

        last_ts = Some(*ts);

        if last_err.as_ref().map_or(false, |l| l == num) || is_int(num) {
            continue;
        }

        last_err = Some(num.clone());

        if first_err.is_some() {
            err_count += 1;
        } else {
            first_err = Some((*ts, num.clone()));
        }
    }

    if let Some(ts) = first_dup {
        let mut s = format!(
            "Stats array for {} of {:?} has duplicate datapoint at ",
            msg, sym
        );

        if let Some(ts) = get_datapoint_timestamp(ts) {
            write!(s, "{ts}").unwrap();
        } else {
            write!(s, "UNIX timestamp {ts}").unwrap();
        }

        s.push('!');

        if dup_count != 0 {
            write!(s, " (plus {dup_count} more)").unwrap();
        }

        warn!("{}", s);
    }

    let (ts, num) = if let Some(pair) = first_err {
        pair
    } else {
        return Ok(());
    };

    int_error(&num, || {
        let mut s = "datapoint at ".to_owned();

        if let Some(ts) = get_datapoint_timestamp(ts) {
            write!(s, "{ts}").unwrap();
        } else {
            write!(s, "UNIX timestamp {ts}").unwrap();
        }

        write!(s, " for {msg} of collection {sym:?}").unwrap();

        if err_count != 0 {
            write!(s, " (plus {err_count} more)").unwrap();
        }

        s
    })?;

    Ok(())
}

fn check_collections(json: &[Collection]) {
    for Collection {
        symbol,
        supply,
        floor,
        listed,
        volume_all,
        ..
    } in json
    {
        let run = || {
            supply.as_ref().map_or(Ok(()), |n| {
                check_int(n, || format!("collection supply for {symbol:?}"))
            })?;
            floor.as_ref().map_or(Ok(()), |n| {
                check_int(n, || format!("collection floor for {symbol:?}"))
            })?;
            listed.as_ref().map_or(Ok(()), |n| {
                check_int(n, || format!("collection listed count for {symbol:?}"))
            })?;
            volume_all.as_ref().map_or(Ok(()), |n| {
                check_int(n, || format!("collection volume for {symbol:?}"))
            })?;

            Result::<_>::Ok(())
        };

        match run() {
            Ok(()) => (),
            Err(e) => error!("{:?}", e),
        }
    }
}

mod insert {
    use holaplex_indexer_dolphin_stats::{MarketStats, MarketStatsResponse};
    use indexer_core::bigdecimal::{BigDecimal, ToPrimitive};

    use super::{
        anyhow, check_stats, debug, dolphin_stats, indexer_core, insert, insert_into,
        market_stats_endpoint, split_stats, update, Arc, Borrowed, Context, DateTime, DolphinStats,
        DolphinStats1D, ExpressionMethods, Pool, QueryDsl, Result, RunQueryDsl, Semaphore, Stats,
        Utc,
    };

    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn calc_percent_change(current: &BigDecimal, previous: &BigDecimal) -> Option<i32> {
        if *previous == 0.into() {
            return None;
        }

        let numerator = current - previous.clone();

        let percentage_change: BigDecimal =
            (numerator / previous.abs()) * <u16 as std::convert::Into<BigDecimal>>::into(100);

        Some(percentage_change.to_f64().unwrap_or_default().floor() as i32)
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn calc_listed_percent_change(current: i64, previous: i64) -> Option<i32> {
        if previous == 0 {
            return None;
        }

        let current = current as f64;
        let previous = previous as f64;

        let numerator = current - previous;

        let percentage_change = (numerator / previous.abs()) * 100.0;

        Some(percentage_change.floor() as i32)
    }

    pub(super) struct Shared {
        pub sem: Semaphore,
        pub dolphin_key: String,
        pub split_info: Stats<u64>,
        pub pool: Pool,
    }

    fn full(
        sym: &str,
        Shared { pool, .. }: &Shared,
        floor: Stats<BigDecimal>,
        listed: Stats<i64>,
        volume: Stats<BigDecimal>,
    ) -> Result<()> {
        let conn = pool.get()?;
        let Stats {
            curr_1d: floor_1d,
            curr_7d: floor_7d,
            curr_30d: floor_30d,
            last_1d: last_floor_1d,
            last_7d: last_floor_7d,
            last_30d: last_floor_30d,
        } = floor;
        let Stats {
            curr_1d: listed_1d,
            curr_7d: listed_7d,
            curr_30d: listed_30d,
            last_1d: last_listed_1d,
            last_7d: last_listed_7d,
            last_30d: last_listed_30d,
        } = listed;
        let Stats {
            curr_1d: volume_1d,
            curr_7d: volume_7d,
            curr_30d: volume_30d,
            last_1d: last_volume_1d,
            last_7d: last_volume_7d,
            last_30d: last_volume_30d,
        } = volume;

        let row = DolphinStats {
            collection_symbol: Borrowed(sym),
            floor_1d: floor_1d.clone(),
            floor_7d: floor_7d.clone(),
            floor_30d: floor_30d.clone(),
            listed_1d,
            listed_7d,
            listed_30d,
            volume_1d: volume_1d.clone(),
            volume_7d: volume_7d.clone(),
            volume_30d: volume_30d.clone(),
            last_floor_1d: last_floor_1d.clone(),
            last_floor_7d: last_floor_7d.clone(),
            last_floor_30d: last_floor_30d.clone(),
            last_listed_1d,
            last_listed_7d,
            last_listed_30d,
            last_volume_1d: last_volume_1d.clone(),
            last_volume_7d: last_volume_7d.clone(),
            last_volume_30d: last_volume_30d.clone(),
            change_floor_1d: calc_percent_change(&floor_1d, &last_floor_1d),
            change_floor_7d: calc_percent_change(&floor_7d, &last_floor_7d),
            change_floor_30d: calc_percent_change(&floor_30d, &last_floor_30d),
            change_volume_1d: calc_percent_change(&volume_1d, &last_volume_1d),
            change_volume_7d: calc_percent_change(&volume_7d, &last_volume_7d),
            change_volume_30d: calc_percent_change(&volume_30d, &last_volume_30d),
            change_listed_1d: calc_listed_percent_change(listed_1d, last_listed_1d),
            change_listed_7d: calc_listed_percent_change(listed_7d, last_listed_7d),
            change_listed_30d: calc_listed_percent_change(listed_30d, last_listed_30d),
        };

        insert_into(dolphin_stats::table)
            .values(&row)
            .on_conflict(dolphin_stats::collection_symbol)
            .do_update()
            .set(&row)
            .execute(&conn)
            .with_context(|| format!("Stats upsert for {sym:?} failed"))?;

        Result::<_>::Ok(())
    }

    fn one_day(
        sym: &str,
        Shared { pool, .. }: &Shared,
        floor: Stats<BigDecimal>,
        listed: Stats<i64>,
        volume: Stats<BigDecimal>,
    ) -> Result<()> {
        let conn = pool.get()?;
        let Stats {
            curr_1d: floor_1d,
            last_1d: last_floor_1d,
            ..
        } = floor;
        let Stats {
            curr_1d: listed_1d,
            last_1d: last_listed_1d,
            ..
        } = listed;
        let Stats {
            curr_1d: volume_1d,
            last_1d: last_volume_1d,
            ..
        } = volume;

        let row = DolphinStats1D {
            collection_symbol: Borrowed(sym),
            floor_1d: floor_1d.clone(),
            listed_1d,
            volume_1d: volume_1d.clone(),
            last_floor_1d: last_floor_1d.clone(),
            last_listed_1d,
            last_volume_1d: last_volume_1d.clone(),
            change_floor_1d: calc_percent_change(&floor_1d, &last_floor_1d),
            change_volume_1d: calc_percent_change(&volume_1d, &last_volume_1d),
            change_listed_1d: calc_listed_percent_change(listed_1d, last_listed_1d),
        };

        update(dolphin_stats::table.filter(dolphin_stats::collection_symbol.eq(sym)))
            .set(row)
            .execute(&conn)
            .with_context(|| format!("One-day stats update for {sym:?} failed"))?;

        Result::<_>::Ok(())
    }

    pub(super) async fn run(
        shared: Arc<Shared>,
        client: reqwest::Client,
        full: bool,
        now: DateTime<Utc>,
        sym: String,
    ) -> Result<()> {
        let insert::Shared {
            ref sem,
            ref dolphin_key,
            ref split_info,
            ..
        } = *shared;

        let permit = sem
            .acquire()
            .await
            .context("Couldn't get semapore permit")?;
        let url = market_stats_endpoint(
            &sym,
            &(now - indexer_core::chrono::Duration::days(if full { 60 } else { 2 })),
            &now,
        )?;
        let json = client
            .get(url.clone())
            .header("Authorization", dolphin_key)
            .header("Content-Type", "application/json")
            .send()
            .await
            .with_context(|| format!("Request failed for {sym:?}"))?
            .json::<MarketStatsResponse>()
            .await
            .with_context(|| format!("Parsing JSON failed for {sym:?}"))?
            .into_inner(|| &url)?;

        std::mem::drop(permit);

        debug!("Completed {:?}", sym);

        #[allow(deprecated)]
        let MarketStats {
            floor_data,
            listed_data,
            volume_data,
            holder_data,
            volume_data_all: _,
        } = json;

        check_stats(&floor_data, "floor data", &sym)?;
        check_stats(&listed_data, "listed data", &sym)?;
        // check_stats(&volume_data, "volume data", &s)?;
        check_stats(&holder_data, "holder data", &sym)?;
        check_stats(&volume_data, "delta volume data", &sym)?;

        let floor = split_stats(split_info, floor_data, |f| {
            f.iter()
                .try_fold(None::<u64>, |s, (_, n)| {
                    n.as_u64()
                        .ok_or_else(|| anyhow!("Invalid number {:?}", n))
                        .map(|n| Some(s.map_or(n, |s| s.min(n))))
                })?
                .unwrap_or(0)
                .try_into()
                .context("Value was too big to store")
        })
        .with_context(|| format!("Error while processing floor data for {sym:?}"))?;

        let listed = split_stats(split_info, listed_data, |f| {
            f.iter()
                .try_fold(None::<u64>, |s, (_, n)| {
                    n.as_u64()
                        .ok_or_else(|| anyhow!("Invalid number {:?}", n))
                        .map(|n| Some(s.map_or(n, |s| s.max(n))))
                })?
                .unwrap_or(0)
                .try_into()
                .context("Value was too big to store")
        })
        .with_context(|| format!("Error while processing listed data for {sym:?}"))?;

        let volume = split_stats(split_info, volume_data, |f| {
            f.first()
                .zip(f.last())
                .map_or(Ok(0), |((_, first), (_, last))| {
                    let first = first
                        .as_u64()
                        .ok_or_else(|| anyhow!("Invalid number {:?}", first))?;
                    let last = last
                        .as_u64()
                        .ok_or_else(|| anyhow!("Invalid number {:?}", last))?;

                    last.checked_sub(first).ok_or_else(|| {
                        anyhow!("Overflow when calculating {:?} - {:?}", last, first)
                    })
                })?
                .try_into()
                .context("Value was too big to store")
        })?;

        tokio::task::spawn_blocking(move || {
            if full {
                insert::full(&sym, &shared, floor, listed, volume)
            } else {
                insert::one_day(&sym, &shared, floor, listed, volume)
            }
        })
        .await??;

        Result::<_>::Ok(())
    }
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts {
            dolphin_key,
            jobs,
            full,
            db,
        } = opts;

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let client = reqwest::Client::new();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let now = Utc::now();

        let url: Url = collections_endpoint().parse()?;
        let json = rt
            .block_on(async {
                client
                    .get(url.clone())
                    .header("Authorization", &dolphin_key)
                    .header("Content-Type", "application/json")
                    .send()
                    .await?
                    .json::<CollectionsResponse>()
                    .await
            })?
            .into_inner(|| &url)?;

        check_collections(&json);

        let symbols: Vec<_> = json
            .iter()
            .filter_map(|c| c.symbol.clone())
            .filter(|s| !s.is_empty())
            .collect();

        let time = Utc::now() - now;

        info!(
            "Collections fetch completed in {}",
            util::duration_hhmmssfff(time)
        );

        if cfg!(debug_assertions) {
            let symbol_set: HashSet<_> = symbols.iter().cloned().collect();

            assert_eq!(symbols.len(), symbol_set.len());
        }

        let now = Utc::now();

        let split_info = Stats {
            curr_1d: get_split(now, Duration::days(1)),
            curr_7d: get_split(now, Duration::days(7)),
            curr_30d: get_split(now, Duration::days(30)),
            last_1d: get_split(now, Duration::days(2)),
            last_7d: get_split(now, Duration::days(14)),
            last_30d: get_split(now, Duration::days(60)),
        };

        rt.block_on(async move {
            let shared = Arc::new(insert::Shared {
                sem: Semaphore::new(jobs),
                dolphin_key,
                split_info,
                pool,
            });

            let handles: Vec<_> = symbols
                .into_iter()
                .map(|sym| {
                    let task = tokio::task::spawn({
                        let client = client.clone();
                        let sym = sym.clone();
                        let shared = Arc::clone(&shared);

                        insert::run(shared, client, full, now, sym)
                    });

                    (sym, task)
                })
                .collect();

            for (sym, handle) in handles {
                match handle
                    .await
                    .with_context(|| format!("Stats task for {sym:?} crashed"))
                    .and_then(|h| h.with_context(|| format!("Stats task for {sym:?} failed")))
                {
                    Ok(()) => (),
                    Err(e) => error!("{:?}", e),
                }
            }
        });

        let time = Utc::now() - now;

        info!(
            "Stats fetch completed in {}",
            util::duration_hhmmssfff(time)
        );

        Ok(())
    });
}
