//! Support code for Dolphin API clients

#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use indexer_core::{prelude::*, url::Url};
use serde_json::Number;

const V3_BASE: &str = "https://app.getdolphin.io/apiv3";

/// API endpoint for the Dolphin collections list
#[inline]
#[must_use]
pub fn collections_endpoint() -> String {
    format!("{V3_BASE}/collections/")
}

/// API endpoint for the Dolphin market stats for the collection identified by
/// `symbol`
///
/// # Errors
/// Returns an error if a valid URL cannot be generated
pub fn market_stats_endpoint<T: TimeZone>(
    symbol: &impl std::fmt::Display,
    start: &DateTime<T>,
    end: &DateTime<T>,
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

/// Wrapper type for Dolphin API responses
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    /// The API returned an error value
    Error {
        /// The returned error information
        error: serde_json::Value,
    },
    /// The API successfully returned the requested model
    Success(T),
}

impl<T> Response<T> {
    /// Converts an API response into an [`anyhow`] result
    ///
    /// # Errors
    /// If the API call returns an error it will be wrapped in `Err`.
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

/// API response for the [collections endpoint](collections_endpoint)
pub type CollectionsResponse = Response<Vec<Collection>>;

/// A single collection from the [collections endpoint](collections_endpoint)
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    /// The collection's identifying symbol
    pub symbol: Option<String>,
    /// The collection's friendly name
    pub name: Option<String>,
    /// The collection's description
    pub description: Option<String>,
    /// The collection's cover image URL
    pub image: Option<String>,
    /// The current supply of the collection
    pub supply: Option<Number>,
    /// The current floor price of the collection
    pub floor: Option<Number>,
    /// The current number of listed items in the collection
    pub listed: Option<Number>,
    /// The total market volume of the collection
    pub volume_all: Option<Number>,
    /// Social links for the collection
    pub external_links: CollectionLinks,
}

/// A set of social links for a collection
#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
pub struct CollectionLinks {
    /// Website for the collection, if any
    pub website: Option<String>,
    /// Discord server for the collection, if any
    pub discord: Option<String>,
    /// Twitter account for the collection, if any
    pub twitter: Option<String>,
}

/// API response for the [market stats endpoint](market_stats_endpoint)
pub type MarketStatsResponse = Response<MarketStats>;

/// Model type for the [market stats endpoint](market_stats_endpoint)
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketStats {
    /// Time-series floor price data
    pub floor_data: Vec<Datapoint>,
    /// Time-series listed item count data
    pub listed_data: Vec<Datapoint>,
    /// Time-series cumulative market volume data
    pub volume_data: Vec<Datapoint>,
    /// Time-series holder count data
    pub holder_data: Vec<Datapoint>,
    /// Time-series delta market volume data
    pub volume_data_all: Vec<Datapoint>,
}

/// A pair of (millisecond UNIX timestamp, value) returned by the [market
/// stats endpoint](market_stats_endpoint)
pub type Datapoint = (u64, Number);

/// Convert a millisecond-precision UNIX timestamp from a [`Datapoint`] to a
/// UTC [`DateTime`]
#[must_use]
pub fn get_datapoint_timestamp(ts: u64) -> Option<DateTime<Utc>> {
    const SPLIT: u64 = 1_000;
    let secs = ts / SPLIT;
    let micros = ts % SPLIT;

    let secs = secs.try_into().ok()?;
    let micros = micros.try_into().ok()?;

    let ts = NaiveDateTime::from_timestamp_opt(secs, micros)?;

    Some(DateTime::from_utc(ts, Utc))
}
