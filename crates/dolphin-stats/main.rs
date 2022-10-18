#![allow(clippy::pedantic, clippy::cargo)]

use std::{borrow::Borrow, sync::Arc};

use indexer_core::{
    chrono::{DateTime, Duration},
    clap,
    clap::Parser,
    db,
    db::{insert_into, models::DolphinStats, tables::dolphin_stats},
    hash::HashSet,
    prelude::*,
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

const COLLECTIONS_ENDPOINT: &str = "https://app.getdolphin.io/apiv3/collections/";

type CollectionsResponse = Vec<Collection>;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Collection {
    symbol: Option<String>,
    name: Option<String>,
    description: Option<String>,
    image: Option<String>,
    supply: Option<Number>,
    floor: Option<Number>,
    listed: Option<Number>,
    #[serde(rename = "volumeAll")]
    volume_all: Option<Number>,
    external_links: CollectionLinks,
}

#[derive(Debug, serde::Deserialize)]
struct CollectionLinks {
    website: Option<String>,
    discord: Option<String>,
    twitter: Option<String>,
}

type Datapoint = (u64, Number);

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MarketStats {
    floor_data: Vec<Datapoint>,
    listed_data: Vec<Datapoint>,
    volume_data: Vec<Datapoint>,
    volume_data_all: Vec<Datapoint>,
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

fn int_error<M: FnOnce() -> D, D: std::fmt::Display>(num: &Number, msg: M) -> Result<()> {
    if cfg!(debug_assertions) {
        error!("Non-integer {:?} found in {}", num, msg());

        Ok(())
    } else {
        Err(anyhow!("Non-integer {:?} found in {}", num, msg()))
    }
}

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
    let mut first = None;
    let mut last_err = None;
    let mut last_ts = None;
    let mut count = 0_u64;
    for pair in nums {
        let (ts, num) = pair.borrow();

        if last_ts.as_ref().map_or(false, |l| l > ts) {
            panic!("Stats array for {} of {:?} was not sorted!", msg, sym);
        }

        last_ts = Some(*ts);

        if last_err.as_ref().map_or(false, |l| l == num) || is_int(num) {
            continue;
        }

        last_err = Some(num.clone());

        if first.is_some() {
            count += 1;
        } else {
            first = Some((*ts, num.clone()));
        }
    }

    let (ts, num) = if let Some(pair) = first {
        pair
    } else {
        return Ok(());
    };

    int_error(&num, || {
        use std::fmt::Write;

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

        if count != 0 {
            write!(s, " (plus {} more)", count).unwrap();
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
        let pool = Arc::new(pool);

        let client = reqwest::Client::new();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let json: CollectionsResponse = rt.block_on(async {
            client
                .get(COLLECTIONS_ENDPOINT)
                .header("Authorization", &dolphin_key)
                .header("Content-Type", "application/json")
                .send()
                .await?
                .json()
                .await
        })?;

        for Collection {
            symbol,
            supply,
            floor,
            listed,
            volume_all,
            ..
        } in &json
        {
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
        }

        let symbols: Vec<_> = json
            .iter()
            .filter_map(|c| c.symbol.clone())
            .filter(|s| !s.is_empty())
            .collect();

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

        let results = rt.block_on(async move {
            let sem = Semaphore::new(jobs);

            futures_util::future::join_all(symbols.into_iter().map(|s| {
                let client = client.clone();
                let dolphin_key = &dolphin_key;
                let sem = &sem;
                let split_info = &split_info;
                let pool = Arc::clone(&pool);

                async move {
                    let permit = sem
                        .acquire()
                        .await
                        .context("Couldn't get semapore permit")?;
                    let json = client
                        .get(market_stats_endpoint(
                            &s,
                            now - indexer_core::chrono::Duration::days(60),
                            now,
                        ))
                        .header("Authorization", dolphin_key)
                        .header("Content-Type", "application/json")
                        .send()
                        .await
                        .with_context(|| format!("Request failed for {:?}", s))?
                        .json()
                        .await
                        .with_context(|| format!("Parsing JSON failed for {:?}", s))?;

                    std::mem::drop(permit);

                    debug!("Completed {:?}", s);

                    let MarketStats {
                        floor_data,
                        listed_data,
                        volume_data,
                        volume_data_all,
                    } = json;

                    check_stats(&floor_data, "floor data", &s)?;
                    check_stats(&listed_data, "listed data", &s)?;
                    check_stats(&volume_data, "volume data", &s)?;
                    check_stats(&volume_data_all, "delta volume data", &s)?;

                    let Stats {
                        curr_1d: floor_1d,
                        curr_7d: floor_7d,
                        curr_30d: floor_30d,
                        last_1d: last_floor_1d,
                        last_7d: last_floor_7d,
                        last_30d: last_floor_30d,
                    } = split_stats(split_info, floor_data, |f| {
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
                    .with_context(|| format!("Error while processing floor data for {:?}", s))?;

                    let Stats {
                        curr_1d: listed_1d,
                        curr_7d: listed_7d,
                        curr_30d: listed_30d,
                        last_1d: last_listed_1d,
                        last_7d: last_listed_7d,
                        last_30d: last_listed_30d,
                    } = split_stats(split_info, listed_data, |f| {
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
                    .with_context(|| format!("Error while processing listed data for {:?}", s))?;

                    let Stats {
                        curr_1d: volume_1d,
                        curr_7d: volume_7d,
                        curr_30d: volume_30d,
                        last_1d: last_volume_1d,
                        last_7d: last_volume_7d,
                        last_30d: last_volume_30d,
                    } = split_stats(split_info, volume_data, |f| {
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
                    })
                    .with_context(|| format!("Error while processing volume data for {:?}", s))?;

                    tokio::task::spawn_blocking(move || {
                        let conn = pool.get()?;

                        let row = DolphinStats {
                            collection_symbol: Borrowed(&s),
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
                            .with_context(|| format!("Stats upsert for {:?} failed", s))?;

                        Result::<_>::Ok(())
                    })
                    .await??;

                    Result::<_>::Ok(())
                }
            }))
            .await
        });

        let time = Utc::now() - now;

        debug!("Received in {}", util::duration_hhmmssfff(time));

        for res in results {
            match res {
                Ok(()) => (),
                Err(e) => error!("{:?}", e),
            }
        }

        Ok(())
    });
}

fn market_stats_endpoint<T: TimeZone>(
    symbol: impl std::fmt::Display,
    start: DateTime<T>,
    end: DateTime<T>,
) -> String {
    let url = format!(
        "https://app.getdolphin.io/apiv3/collections/marketStats/&symbol={}&timestamp_from={}&timestamp_to={}",
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

    url
}
