#![allow(clippy::pedantic, clippy::cargo)]

use indexer::prelude::*;
use indexer_core::{chrono::DateTime, clap, clap::Parser, db};

#[derive(Debug, Parser)]
struct Opts {
    /// Dolphin API key
    #[clap(long, env)]
    dolphin_key: String,

    #[clap(flatten)]
    db: db::ConnectArgs,
}

const COLLECTIONS_ENDPOINT: &str = "https://app.getdolphin.io/apiv3/collections/";

type CollectionsResponse = Vec<Collection>;

#[derive(Debug, serde::Deserialize)]
struct Collection {
    symbol: Option<String>,
    name: String,
    description: Option<String>,
    image: Option<String>,
    supply: Option<String>,
    floor: Option<String>,
    listed: Option<String>,
    // can't use rename_all because external_links is still snake_case
    #[serde(rename = "volumeAll")]
    volume_all: Option<f64>,
    external_links: CollectionLinks,
}

#[derive(Debug, serde::Deserialize)]
struct CollectionLinks {
    website: Option<String>,
    discord: Option<String>,
    twitter: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketStats {
    floor_data: Vec<(u64, f64)>,
    listed_data: Vec<(u64, f64)>,
    volume_data: Vec<(u64, f64)>,
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);

        let Opts { dolphin_key, db } = opts;

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let conn = pool.get()?;
        let client = reqwest::Client::new();
        let rt = tokio::runtime::Builder::new_current_thread()
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

        debug!("{:#?}", json);

        let now = Utc::now();
        let json: MarketStats = rt.block_on(async {
            client
                .get(market_stats_endpoint(
                    "zukustags",
                    now - indexer_core::chrono::Duration::hours(2),
                    now,
                ))
                .header("Authorization", dolphin_key)
                .header("Content-Type", "application/json")
                .send()
                .await?
                .json()
                .await
        })?;

        debug!("{:#?}", json);

        Ok(())
    });
}

fn market_stats_endpoint<T: TimeZone>(
    symbol: impl std::fmt::Display,
    start: DateTime<T>,
    end: DateTime<T>,
) -> String {
    let url = format!(
        "https://app.getdolphin.io/apiv3/collections/marketStats/&symbol={}&timestamp_from={}&timestamp_end={}",
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
