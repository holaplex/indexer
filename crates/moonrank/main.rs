#![allow(clippy::pedantic, clippy::cargo)]

use std::collections::HashMap;

use futures_util::StreamExt;
use indexer::prelude::*;
use indexer_core::{
    clap,
    clap::Parser,
    db::{
        self, delete, insert_into,
        models::{Collection as DbCollection, CollectionMint, CollectionMintAttribute},
        tables::{collection_mint_attributes, collection_mints, collections},
        Pool,
    },
    num_cpus,
    util::unix_timestamp_unsigned,
};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Collection {
    id: String,
    image: String,
    name: String,
    description: String,
    verified_collection_address: Option<String>,
    pieces: u64,
    verified: bool,
    metadata: Metadata,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    private: bool,
    shadow: bool,
    access_key: String,
    go_live_at: String,
    api_block: bool,
    x_collection_metadata: Option<XCollectionMetadata>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct XCollectionMetadata {
    derived: bool,
    last_modified: String,
    #[serde(rename = "x.id")]
    x_id: String,
    #[serde(rename = "x.supply")]
    x_supply: u64,
    #[serde(rename = "x.url.web")]
    x_url_web: Option<String>,
    #[serde(rename = "x.url.twitter")]
    x_url_twitter: Option<String>,
    #[serde(rename = "x.url.discord")]
    x_url_discord: Option<String>,
    #[serde(rename = "x.market.magiceden.id")]
    x_market_magiceden_id: Option<String>,
    #[serde(rename = "x.market.solanart.id")]
    x_market_solanart_id: Option<String>,
    #[serde(rename = "x.market.hyperspace.id")]
    x_market_hyperspace_id: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Crawl {
    id: String,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    first_mint_ts: u64,
    last_mint_ts: u64,
    first_mint: String,
    last_mint: String,
    expected_pieces: u64,
    seen_pieces: u64,
    last_crawl_id: u128,
    complete: bool,
    blocked: bool,
    unblocked_at_ts: Option<u64>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    #[serde(rename = "Collection")]
    collection: Collection,
    #[serde(rename = "Crawl")]
    crawl: Crawl,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CollectionMints {
    partial: Option<bool>,
    collection: Option<Value>,
    crawl: Option<Crawl>,
    mints: Vec<Mint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Mint {
    crawl_id: u128,
    mint: String,
    name: String,
    image: String,
    created: u64,
    rank: u64,
    rarity: f64,
    rank_explain: Vec<Attribute>,
    filtered_rank_explain: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Attribute {
    attribute: String,
    value: String,
    value_perc: f64,
    times_seen: u64,
    total_seen: u64,
}

#[derive(Debug, Parser)]
struct Opts {
    /// MoonRank RPC endpoint
    #[clap(long, env)]
    moonrank_endpoint: String,

    #[clap(flatten)]
    db: db::ConnectArgs,
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();

        let Opts {
            moonrank_endpoint,
            db,
        } = opts;

        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(num_cpus::get())
            .build()?;

        runtime.block_on(get_collections(moonrank_endpoint, pool))
    });
}

async fn get_collections(endpoint: String, conn: Pool) -> Result<()> {
    let collection_list = url::Url::parse(&endpoint)?.join("../collection_list")?;

    let http = reqwest::Client::builder().build()?;

    let bytes = http.get(collection_list).send().await?.bytes().await?;

    let collections: Vec<Data> = serde_json::from_slice(&bytes)?;

    futures_util::stream::iter(collections.into_iter().map(|data| {
        tokio::spawn(upsert_collection_data(
            endpoint.clone(),
            data,
            conn.clone(),
            http.clone(),
        ))
    }))
    .buffer_unordered(num_cpus::get())
    .collect::<Vec<_>>()
    .await;

    Ok(())
}

async fn upsert_collection_data(
    endpoint: String,
    json: Data,
    pool: Pool,
    http: reqwest::Client,
) -> Result<()> {
    let conn = pool.get()?;
    let collection_id = json.collection.id;

    let indexed_timestamp: Option<NaiveDateTime> = collections::table
        .filter(collections::id.eq(collection_id.clone()))
        .select(collections::updated_at)
        .first(&conn)
        .optional()?;

    // can use crawl_id instead of updated timestamp

    if indexed_timestamp == Some(json.crawl.updated.naive_utc()) {
        return Ok(());
    }

    let collection = DbCollection {
        id: Owned(collection_id.clone()),
        image: Owned(json.collection.image),
        name: Owned(json.collection.name),
        description: Owned(json.collection.description),
        verified_collection_address: json.collection.verified_collection_address.map(Owned),
        pieces: json.collection.pieces.try_into()?,
        verified: json.collection.verified,
        created_at: json.crawl.created.naive_utc(),
        updated_at: json.crawl.updated.naive_utc(),
    };

    insert_into(collections::table)
        .values(&collection)
        .on_conflict(collections::id)
        .do_update()
        .set(&collection)
        .execute(&conn)?;

    let collection_mints = url::Url::parse(&endpoint)?
        .join("../mints/")?
        .join(&collection_id)?;

    let bytes = http.get(collection_mints).send().await?.bytes().await?;

    let mints_json: CollectionMints = serde_json::from_slice(&bytes)?;

    for mint in mints_json.mints {
        let collection_id = collection_id.clone();
        let values = CollectionMint {
            collection_id: Owned(collection_id),
            mint: Owned(mint.mint.clone().to_string()),
            name: Owned(mint.name),
            image: Owned(mint.image),
            created_at: unix_timestamp_unsigned(mint.created)?,
            rank: mint.rank.try_into()?,
            rarity: mint.rarity.try_into()?,
        };

        insert_into(collection_mints::table)
            .values(&values)
            .on_conflict((collection_mints::collection_id, collection_mints::mint))
            .do_update()
            .set(&values)
            .execute(&conn)?;

        for attribute in mint.rank_explain {
            let row = CollectionMintAttribute {
                mint: Owned(mint.mint.clone().to_string()),
                attribute: Owned(attribute.attribute.to_string()),
                value: Owned(attribute.value.to_string()),
                value_perc: attribute.value_perc.try_into()?,
            };

            delete(
                collection_mint_attributes::table
                    .filter(collection_mint_attributes::mint.eq(mint.mint.to_string())),
            )
            .execute(&conn)?;

            insert_into(collection_mint_attributes::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(&conn)?;
        }
    }

    Ok(())
}
