#![allow(clippy::pedantic, clippy::cargo)]

use std::{borrow::Borrow, sync::Arc};

use indexer_core::{
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
struct Opts {
    /// Dolphin API key
    #[clap(long, env)]
    dolphin_key: String,

    /// Maximum number of concurrent requests
    #[clap(short, long, env, default_value_t = 192)]
    jobs: usize,

    /// Request 60 days of data to compute all statistics
    ///
    /// By default only two days are requested to update the day-over-day values
    #[clap(short, long, env)]
    full: bool,

    #[clap(flatten)]
    db: db::ConnectArgs,
}

const V3_BASE: &str = "https://app.getdolphin.io/apiv3";

#[inline]
fn collections_endpoint() -> String {
    format!("{}/collections/", V3_BASE)
}

fn market_stats_endpoint<T: TimeZone>(
    symbol: impl std::fmt::Display,
    start: DateTime<T>,
    end: DateTime<T>,
) -> Result<Url> {
    let url = format!(
        "{}/collections/marketStats/&symbol={}&timestamp_from={}&timestamp_to={}",
        V3_BASE,
        percent_encoding::utf8_percent_encode(
            &symbol.to_string(),
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            &start.timestamp().to_string(),
            percent_encoding::NON_ALPHANUMERIC
        ),
        percent_encoding::utf8_percent_encode(
            &end.timestamp().to_string(),
            percent_encoding::NON_ALPHANUMERIC
        ),
    );

    debug!("Market stats URL: {:?}", url);

    url.parse().map_err(Into::into)
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Response<T> {
    Error { error: serde_json::Value },
    Success(T),
}

impl<T> Response<T> {
    fn into_inner<'a>(self, url: impl FnOnce() -> &'a Url) -> Result<T> {
        match self {
            Self::Error { error } => Err(anyhow!(
                "API call for {:?} returned error: {:?}",
                url().as_str(),
                error
            )),
            Self::Success(s) => Ok(s),
        }
    }
}

type CollectionsResponse = Response<Vec<Collection>>;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Collection {
    symbol: Option<String>,
    #[allow(unused)]
    name: Option<String>,
    #[allow(unused)]
    description: Option<String>,
    #[allow(unused)]
    image: Option<String>,
    supply: Option<Number>,
    floor: Option<Number>,
    listed: Option<Number>,
    #[serde(rename = "volumeAll")]
    volume_all: Option<Number>,
    #[allow(unused)]
    external_links: CollectionLinks,
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct CollectionLinks {
    website: Option<String>,
    discord: Option<String>,
    twitter: Option<String>,
}

type MarketStatsResponse = Response<MarketStats>;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MarketStats {
    floor_data: Vec<Datapoint>,
    listed_data: Vec<Datapoint>,
    #[deprecated = "Use volume_data_all"]
    #[allow(unused)]
    volume_data: Vec<Datapoint>,
    holder_data: Vec<Datapoint>,
    volume_data_all: Vec<Datapoint>,
}

type Datapoint = (u64, Number);

#[derive(Debug, Clone, Copy)]
struct Stats<T> {
    curr_1d: T,
    curr_7d: T,
    curr_30d: T,
    last_1d: T,
    last_7d: T,
    last_30d: T,
}

#[inline]
// Panics if your NFT price activity occurred before Jan 1 1970.  lol.
fn get_split(now: DateTime<Utc>, offset: Duration) -> u64 {
    (now - offset).timestamp_millis().try_into().unwrap()
}

#[inline]
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

        if last_ts.as_ref().map_or(false, |l| l > ts) {
            panic!("Stats array for {} of {:?} was not sorted!", msg, sym);
        }

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

        const SPLIT: u64 = 1_000;
        let secs = ts / SPLIT;
        let micros = ts % SPLIT;

        if let Some(ts) = secs
            .try_into()
            .ok()
            .and_then(|s| NaiveDateTime::from_timestamp_opt(s, micros.try_into().ok()?))
        {
            write!(s, "{}", ts).unwrap();
        } else {
            write!(s, "UNIX timestamp {}", ts).unwrap();
        }

        s.push('!');

        if dup_count != 0 {
            write!(s, " (plus {} more)", dup_count).unwrap();
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

        const SPLIT: u64 = 1_000;
        let secs = ts / SPLIT;
        let micros = ts % SPLIT;

        if let Some(ts) = secs
            .try_into()
            .ok()
            .and_then(|s| NaiveDateTime::from_timestamp_opt(s, micros.try_into().ok()?))
        {
            write!(s, "{}", ts).unwrap();
        } else {
            write!(s, "UNIX timestamp {}", ts).unwrap();
        }

        write!(s, " for {} of collection {:?}", msg, sym).unwrap();

        if err_count != 0 {
            write!(s, " (plus {} more)", err_count).unwrap();
        }

        s
    })?;

    Ok(())
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

        for Collection {
            symbol,
            supply,
            floor,
            listed,
            volume_all,
            ..
        } in &json
        {
            let run = || {
                supply.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection supply for {:?}", symbol))
                })?;
                floor.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection floor for {:?}", symbol))
                })?;
                listed.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection listed count for {:?}", symbol))
                })?;
                volume_all.as_ref().map_or(Ok(()), |n| {
                    check_int(n, || format!("collection volume for {:?}", symbol))
                })?;

                Result::<_>::Ok(())
            };

            match run() {
                Ok(()) => (),
                Err(e) => error!("{:?}", e),
            }
        }

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
            struct Shared {
                sem: Semaphore,
                dolphin_key: String,
                split_info: Stats<u64>,
                pool: Pool,
            }

            fn insert_full(
                sym: &str,
                Shared { pool, .. }: &Shared,
                floor: Stats<i64>,
                listed: Stats<i64>,
                volume: Stats<i64>,
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
                    floor_1d,
                    floor_7d,
                    floor_30d,
                    listed_1d,
                    listed_7d,
                    listed_30d,
                    volume_1d,
                    volume_7d,
                    volume_30d,
                    last_floor_1d,
                    last_floor_7d,
                    last_floor_30d,
                    last_listed_1d,
                    last_listed_7d,
                    last_listed_30d,
                    last_volume_1d,
                    last_volume_7d,
                    last_volume_30d,
                };

                insert_into(dolphin_stats::table)
                    .values(&row)
                    .on_conflict(dolphin_stats::collection_symbol)
                    .do_update()
                    .set(&row)
                    .execute(&conn)
                    .with_context(|| format!("Stats upsert for {:?} failed", sym))?;

                Result::<_>::Ok(())
            }

            fn insert_1d(
                sym: &str,
                Shared { pool, .. }: &Shared,
                floor: Stats<i64>,
                listed: Stats<i64>,
                volume: Stats<i64>,
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
                    floor_1d,
                    listed_1d,
                    volume_1d,
                    last_floor_1d,
                    last_listed_1d,
                    last_volume_1d,
                };

                update(dolphin_stats::table.filter(dolphin_stats::collection_symbol.eq(sym)))
                    .set(row)
                    .execute(&conn)
                    .with_context(|| format!("One-day stats update for {:?} failed", sym))?;

                Result::<_>::Ok(())
            }

            let shared = Arc::new(Shared {
                sem: Semaphore::new(jobs),
                dolphin_key,
                split_info,
                pool,
            });

            let handles: Vec<_> = symbols
                .into_iter()
                .map(|s| {
                    let task = tokio::task::spawn({
                        let client = client.clone();
                        let s = s.clone();
                        let shared = Arc::clone(&shared);

                        async move {
                            let Shared {
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
                                &s,
                                now - indexer_core::chrono::Duration::days(if full {
                                    60
                                } else {
                                    2
                                }),
                                now,
                            )?;
                            let json = client
                                .get(url.clone())
                                .header("Authorization", dolphin_key)
                                .header("Content-Type", "application/json")
                                .send()
                                .await
                                .with_context(|| format!("Request failed for {:?}", s))?
                                .json::<MarketStatsResponse>()
                                .await
                                .with_context(|| format!("Parsing JSON failed for {:?}", s))?
                                .into_inner(|| &url)?;

                            std::mem::drop(permit);

                            debug!("Completed {:?}", s);

                            #[allow(deprecated)]
                            let MarketStats {
                                floor_data,
                                listed_data,
                                volume_data: _,
                                holder_data,
                                volume_data_all,
                            } = json;

                            check_stats(&floor_data, "floor data", &s)?;
                            check_stats(&listed_data, "listed data", &s)?;
                            // check_stats(&volume_data, "volume data", &s)?;
                            check_stats(&holder_data, "holder data", &s)?;
                            check_stats(&volume_data_all, "delta volume data", &s)?;

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
                            .with_context(|| {
                                format!("Error while processing floor data for {:?}", s)
                            })?;

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
                            .with_context(|| {
                                format!("Error while processing listed data for {:?}", s)
                            })?;

                            let volume =
                                split_stats(split_info, volume_data_all, |f| {
                                    f.iter()
                                        .try_fold(None::<u64>, |s, (_, n)| {
                                            let n = n
                                                .as_u64()
                                                .ok_or_else(|| anyhow!("Invalid number {:?}", n))?;

                                            let next =
                                                s.map_or(Ok(n), |s| {
                                                    s.checked_add(n).ok_or_else(|| {
                                        anyhow!("Overflow when calculationg {:?} + {:?}", s, n)
                                    })
                                                })?;

                                            Result::<_>::Ok(Some(next))
                                        })?
                                        .unwrap_or(0)
                                        .try_into()
                                        .context("Value was too big to store")
                                })
                                .with_context(|| {
                                    format!("Error while processing volume data for {:?}", s)
                                })?;

                            tokio::task::spawn_blocking(move || {
                                if full {
                                    insert_full(&s, &*shared, floor, listed, volume)
                                } else {
                                    insert_1d(&s, &*shared, floor, listed, volume)
                                }
                            })
                            .await??;

                            Result::<_>::Ok(())
                        }
                    });

                    (s, task)
                })
                .collect();

            for (sym, handle) in handles {
                match handle
                    .await
                    .with_context(|| format!("Stats task for {:?} crashed", sym))
                    .and_then(|h| h.with_context(|| format!("Stats task for {:?} failed", sym)))
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
