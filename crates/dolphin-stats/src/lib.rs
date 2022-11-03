#![allow(clippy::pedantic, clippy::cargo, missing_docs)]

use std::borrow::Borrow;

use indexer_core::{
    self,
    chrono::{DateTime, Duration},
    clap,
    clap::Parser,
    db,
    prelude::*,
    url::Url,
};
use serde_json::Number;

#[derive(Debug, Parser)]
pub struct Opts {
    /// Dolphin API key
    #[clap(long, env)]
    pub dolphin_key: String,

    /// Maximum number of concurrent requests
    #[clap(short, long, env, default_value_t = 192)]
    pub jobs: usize,

    /// Request 60 days of data to compute all statistics
    ///
    /// By default only two days are requested to update the day-over-day values
    #[clap(short, long, env)]
    pub full: bool,

    #[clap(flatten)]
    pub db: db::ConnectArgs,
}

const V3_BASE: &str = "https://app.getdolphin.io/apiv3";

#[inline]
pub fn collections_endpoint() -> String {
    format!("{}/collections/", V3_BASE)
}

pub fn market_stats_endpoint<T: TimeZone>(
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
pub enum Response<T> {
    Error { error: serde_json::Value },
    Success(T),
}

impl<T> Response<T> {
    pub fn into_inner<'a>(self, url: impl FnOnce() -> &'a Url) -> Result<T> {
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

pub type CollectionsResponse = Response<Vec<Collection>>;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    pub symbol: Option<String>,
    #[allow(unused)]
    pub name: Option<String>,
    #[allow(unused)]
    pub description: Option<String>,
    #[allow(unused)]
    pub image: Option<String>,
    pub supply: Option<Number>,
    pub floor: Option<Number>,
    pub listed: Option<Number>,
    #[serde(rename = "volumeAll")]
    pub volume_all: Option<Number>,
    #[allow(unused)]
    pub external_links: CollectionLinks,
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct CollectionLinks {
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
}

pub type MarketStatsResponse = Response<MarketStats>;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketStats {
    pub floor_data: Vec<Datapoint>,
    pub listed_data: Vec<Datapoint>,
    #[deprecated = "Use volume_data_all"]
    #[allow(unused)]
    pub volume_data: Vec<Datapoint>,
    pub holder_data: Vec<Datapoint>,
    pub volume_data_all: Vec<Datapoint>,
}

pub type Datapoint = (u64, Number);

#[derive(Debug, Clone, Copy)]
pub struct Stats<T> {
    pub curr_1d: T,
    pub curr_7d: T,
    pub curr_30d: T,
    pub last_1d: T,
    pub last_7d: T,
    pub last_30d: T,
}

#[inline]
// Panics if your NFT price activity occurred before Jan 1 1970.  lol.
pub fn get_split(now: DateTime<Utc>, offset: Duration) -> u64 {
    (now - offset).timestamp_millis().try_into().unwrap()
}

#[inline]
pub fn slice_stats(stats: &[Datapoint], start: u64, mid: u64) -> (&[Datapoint], &[Datapoint]) {
    let start_i = stats.partition_point(|(t, _)| *t < start);
    let mid_i = stats.partition_point(|(t, _)| *t < mid);

    debug_assert!(start_i <= mid_i);

    stats[start_i..].split_at(mid_i - start_i)
}

pub fn split_stats<T, E: Into<Error>>(
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
pub fn calc_percent_change(current: i64, previous: i64) -> Option<i32> {
    if previous == 0 {
        return None;
    }

    let current = current as f64;
    let previous = previous as f64;

    let numerator = current - previous;

    let percentage_change = (numerator / previous.abs()) * 100.0;

    Some(percentage_change.floor() as i32)
}

#[inline]
pub fn is_int(n: &Number) -> bool {
    n.is_i64() || n.is_u64()
}

#[inline]
pub fn int_error<M: FnOnce() -> D, D: std::fmt::Display>(num: &Number, msg: M) -> Result<()> {
    Err(anyhow!("Non-integer {:?} found in {}", num, msg()))
}

#[inline]
pub fn check_int<M: FnOnce() -> D, D: std::fmt::Display>(num: &Number, msg: M) -> Result<()> {
    if is_int(num) {
        return Ok(());
    }

    int_error(num, msg)
}

pub fn get_datapoint_timestamp(ts: u64) -> Option<DateTime<Utc>> {
    const SPLIT: u64 = 1_000;
    let secs = ts / SPLIT;
    let micros = ts % SPLIT;

    let secs = secs.try_into().ok()?;
    let micros = micros.try_into().ok()?;

    let ts = NaiveDateTime::from_timestamp_opt(secs, micros)?;

    Some(DateTime::from_utc(ts, Utc))
}

pub fn check_stats<N: IntoIterator>(
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

        if let Some(ts) = get_datapoint_timestamp(ts) {
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

        if let Some(ts) = get_datapoint_timestamp(ts) {
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
