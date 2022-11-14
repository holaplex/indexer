//! Collection data scraper using the Moonrank API

#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::collections::HashMap;

use futures_util::StreamExt;
use indexer::search_dispatch;
use indexer_core::{
    assets::{proxy_url, AssetIdentifier, AssetProxyArgs},
    clap,
    clap::Parser,
    db::{
        self, delete, insert_into,
        models::{self, Collection as DbCollection, CollectionMint, CollectionMintAttribute},
        select,
        tables::{
            attribute_group_variants, attribute_groups, collection_mint_attributes,
            collection_mints, collections,
        },
        Pool,
    },
    num_cpus,
    prelude::*,
    util::unix_timestamp_unsigned,
};
use indexer_rabbitmq::{search_indexer, suffix::Suffix};
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
    go_live_at: DateTime<Utc>,
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
#[command(about, version, long_about = None)]
struct Opts {
    /// MoonRank RPC endpoint
    #[arg(long, env)]
    moonrank_endpoint: String,

    /// MoonRank Authorization token
    #[arg(long, env)]
    moonrank_auth: String,

    #[command(flatten)]
    search: search_dispatch::Args,

    #[command(flatten)]
    db: db::ConnectArgs,

    #[command(flatten)]
    asset_proxy: AssetProxyArgs,

    /// The address of an AMQP server to connect to
    #[arg(long, env)]
    amqp_url: String,

    /// The ID of the indexer sending events to listen for
    #[arg(long, env)]
    sender: String,

    #[command(flatten)]
    queue_suffix: Suffix,
}

fn main() {
    indexer_core::run(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(num_cpus::get())
            .build()?;

        runtime.block_on(process())
    });
}

async fn process() -> Result<()> {
    let opts = Opts::parse();

    debug!("{:#?}", opts);

    let Opts {
        moonrank_endpoint,
        moonrank_auth,
        search,
        db,
        amqp_url,
        sender,
        queue_suffix,
        asset_proxy,
    } = opts;

    let db::ConnectResult {
        pool,
        ty: _,
        migrated: _,
    } = db::connect(db, db::ConnectMode::Write { migrate: false })?;

    let receiver = match queue_suffix {
        Suffix::Debug(ref s) => s.clone(),
        _ => sender.clone(),
    };

    let rabbitmq_conn = indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;

    let search = search_dispatch::Client::new(
        &rabbitmq_conn,
        search_indexer::QueueType::new(&receiver, &queue_suffix)?,
        search,
    )
    .await?;

    let http = reqwest::Client::builder().build()?;

    let collection_list = url::Url::parse(&moonrank_endpoint)?.join("../collection_list")?;

    let collections = http
        .get(collection_list)
        .header("mr-api-pubkey", moonrank_auth.clone())
        .send()
        .await?
        .json::<Vec<Data>>()
        .await?;

    futures_util::stream::iter(collections.clone().into_iter().map(|data| {
        tokio::spawn(upsert_collection_data(
            moonrank_endpoint.clone(),
            moonrank_auth.clone(),
            data,
            pool.clone(),
            http.clone(),
        ))
    }))
    .buffer_unordered(num_cpus::get())
    .collect::<Vec<_>>()
    .await;

    dispatch_documents(collections, search, asset_proxy).await?;

    Ok(())
}

async fn upsert_collection_data(
    endpoint: String,
    auth: String,
    json: Data,
    pool: Pool,
    http: reqwest::Client,
) -> Result<()> {
    let conn = pool.get()?;
    let collection_id = json.collection.id;

    let mut discord_url = None;
    let mut twitter_url = None;
    let mut website_url = None;
    let mut magic_eden_id = None;

    let indexed_timestamp: Option<NaiveDateTime> = collections::table
        .filter(collections::id.eq(collection_id.clone()))
        .select(collections::updated_at)
        .first(&conn)
        .optional()?;

    // can use crawl_id instead of updated timestamp

    if indexed_timestamp == Some(json.crawl.updated.naive_utc()) {
        return Ok(());
    }

    if let Some(metadata) = json.collection.metadata.x_collection_metadata {
        discord_url = metadata.x_url_discord.map(Owned);
        twitter_url = metadata.x_url_twitter.map(Owned);
        website_url = metadata.x_url_web.map(Owned);
        magic_eden_id = metadata.x_market_magiceden_id.map(Owned);
    }

    let collection = DbCollection {
        id: Owned(collection_id.clone()),
        image: Owned(json.collection.image),
        name: Owned(json.collection.name),
        description: Owned(json.collection.description),
        twitter_url,
        discord_url,
        website_url,
        magic_eden_id,
        verified_collection_address: json.collection.verified_collection_address.map(Owned),
        pieces: json.collection.pieces.try_into()?,
        verified: json.collection.verified,
        go_live_at: json.collection.metadata.go_live_at.naive_utc(),
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

    let bytes = http
        .get(collection_mints)
        .header("mr-api-pubkey", auth)
        .send()
        .await?
        .bytes()
        .await?;

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

        delete(
            collection_mint_attributes::table
                .filter(collection_mint_attributes::mint.eq(mint.mint.to_string())),
        )
        .execute(&conn)?;

        for attribute in mint.rank_explain {
            let collection_id = values.collection_id.clone();

            let total_seen: i64 = attribute.total_seen.try_into()?;
            let times_seen: i64 = attribute.times_seen.try_into()?;

            let attribute_group_exists = select(exists(
                attribute_groups::table.filter(
                    attribute_groups::collection_id
                        .eq(collection_id.clone())
                        .and(attribute_groups::trait_type.eq(attribute.attribute.clone()))
                        .and(attribute_groups::total_count.eq(total_seen)),
                ),
            ))
            .get_result::<bool>(&conn)?;

            let attribute_group_variant_exists = select(exists(
                attribute_group_variants::table.filter(
                    attribute_group_variants::collection_id
                        .eq(collection_id.clone())
                        .and(attribute_group_variants::trait_type.eq(attribute.attribute.clone()))
                        .and(attribute_group_variants::count.eq(times_seen)),
                ),
            ))
            .get_result::<bool>(&conn)?;

            if !attribute_group_exists {
                let attribute_group = models::AttributeGroup {
                    collection_id: collection_id.clone(),
                    trait_type: Owned(attribute.attribute.clone()),
                    total_count: attribute.total_seen.try_into()?,
                };

                insert_into(attribute_groups::table)
                    .values(attribute_group.clone())
                    .on_conflict((
                        attribute_groups::collection_id,
                        attribute_groups::trait_type,
                    ))
                    .do_update()
                    .set(attribute_group)
                    .execute(&conn)?;
            }

            if !attribute_group_variant_exists {
                let attribute_group_variant = models::AttributeGroupVariant {
                    collection_id: collection_id.clone(),
                    trait_type: Owned(attribute.attribute.clone()),
                    value: Owned(attribute.value.clone()),
                    count: attribute.times_seen.try_into()?,
                };

                insert_into(attribute_group_variants::table)
                    .values(attribute_group_variant.clone())
                    .on_conflict((
                        attribute_group_variants::collection_id,
                        attribute_group_variants::trait_type,
                        attribute_group_variants::value,
                    ))
                    .do_update()
                    .set(attribute_group_variant)
                    .execute(&conn)?;
            }

            let row = CollectionMintAttribute {
                mint: Owned(mint.mint.clone().to_string()),
                attribute: Owned(attribute.attribute.clone()),
                value: Owned(attribute.value.to_string()),
                value_perc: attribute.value_perc.try_into()?,
            };

            insert_into(collection_mint_attributes::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(&conn)?;
        }
    }

    Ok(())
}

async fn dispatch_documents(
    collections: Vec<Data>,
    search: search_dispatch::Client,
    asset_proxy: AssetProxyArgs,
) -> Result<()> {
    let mut discord_url = None;
    let mut twitter_url = None;
    let mut website_url = None;
    let mut magic_eden_id = None;

    for c in collections {
        if let Some(metadata) = c.collection.metadata.x_collection_metadata {
            discord_url = metadata.x_url_discord;
            twitter_url = metadata.x_url_twitter;
            website_url = metadata.x_url_web;
            magic_eden_id = metadata.x_market_magiceden_id;
        }

        let image = url::Url::parse(&c.collection.image)
            .ok()
            .and_then(|u| {
                proxy_url(
                    &asset_proxy,
                    &AssetIdentifier::new(&u),
                    Some(("width", "200")),
                )
                .map(|o| o.map(|u| u.to_string()))
                .transpose()
            })
            .transpose()?
            .unwrap_or(c.collection.image);

        search
            .upsert_mr_collection(
                false,
                c.collection.id,
                search_dispatch::MRCollectionDocument {
                    name: c.collection.name,
                    image,
                    magic_eden_id: magic_eden_id.clone(),
                    verified_collection_address: c.collection.verified_collection_address,
                    twitter_url: twitter_url.clone(),
                    discord_url: discord_url.clone(),
                    website_url: website_url.clone(),
                },
            )
            .await?;
    }

    Ok(())
}
