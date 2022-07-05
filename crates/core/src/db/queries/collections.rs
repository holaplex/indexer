//! Query utilities for collections.

use anyhow::Context;
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg,
    sql_types::{Array, Integer, Nullable, Text, Timestamp},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{custom_types::OrderDirection, models::Nft, Connection},
    error::Result,
};

/// Query collections ordered by volume
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn by_volume(
    conn: &Connection,
    addresses: impl ToSql<Nullable<Array<Text>>, Pg>,
    order_direction: OrderDirection,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Nft>> {
    diesel::sql_query(make_by_volume_query_string(order_direction))
        .bind(addresses)
        .bind::<Timestamp, _>(start_date.naive_utc())
        .bind::<Timestamp, _>(end_date.naive_utc())
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load collections by volume")
}

fn make_by_volume_query_string(order_direction: OrderDirection) -> String {
    format!(
        r"
        WITH volume_table (collection, volume) as (
            SELECT metadata_collection_keys.collection_address AS collection, SUM(purchases.price) AS volume
                FROM purchases
                INNER JOIN metadatas on (purchases.metadata = metadatas.address)
                INNER JOIN metadata_collection_keys on (metadatas.address = metadata_collection_keys.metadata_address)
                WHERE
                    ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
                    AND purchases.created_at >= $2
                    AND purchases.created_at <= $3
                GROUP BY metadata_collection_keys.collection_address
                ORDER BY volume {order_direction}
                LIMIT $4
                OFFSET $5
        ) SELECT
            metadatas.address,
            metadatas.name,
            metadatas.seller_fee_basis_points,
            metadatas.update_authority_address,
            metadatas.mint_address,
            metadatas.primary_sale_happened,
            metadatas.uri,
            metadatas.slot,
            metadata_jsons.description,
            metadata_jsons.image,
            metadata_jsons.category,
            metadata_jsons.model
        FROM metadata_jsons, volume_table, metadatas
        WHERE
            volume_table.collection = metadatas.mint_address
            AND metadatas.address = metadata_jsons.metadata_address
    -- $1: addresses::text[]
    -- $2: start date::timestamp
    -- $3: end date::timestamp
    -- $4: limit::integer
    -- $5: offset::integer",
        order_direction = order_direction
    )
}

/// Query collections ordered by market cap
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn by_market_cap(
    conn: &Connection,
    addresses: impl ToSql<Nullable<Array<Text>>, Pg>,
    order_direction: OrderDirection,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Nft>> {
    diesel::sql_query(make_by_market_cap_query_string(order_direction))
        .bind(addresses)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load collections by market cap")
}

fn make_by_market_cap_query_string(order_direction: OrderDirection) -> String {
    format!(
        r"
        WITH market_cap_table (collection, market_cap) as (
            SELECT metadata_collection_keys.collection_address AS collection, MIN(listings.price) * COUNT( DISTINCT metadata_collection_keys.metadata_address) AS market_cap
            FROM listings
            INNER JOIN metadatas on (listings.metadata = metadatas.address)
            INNER JOIN metadata_collection_keys on (metadatas.address = metadata_collection_keys.metadata_address)
            INNER JOIN auction_houses on (listings.auction_house = auction_houses.address)
            WHERE
                ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
                AND auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
                AND listings.created_at >= $2
                AND listings.created_at <= $3
            GROUP BY metadata_collection_keys.collection_address
            ORDER BY market_cap {order_direction}
            LIMIT $4
            OFFSET $5
        ) SELECT
            metadatas.address,
            metadatas.name,
            metadatas.seller_fee_basis_points,
            metadatas.update_authority_address,
            metadatas.mint_address,
            metadatas.primary_sale_happened,
            metadatas.uri,
            metadatas.slot,
            metadata_jsons.description,
            metadata_jsons.image,
            metadata_jsons.category,
            metadata_jsons.model
        FROM metadata_jsons, market_cap_table, metadatas
        WHERE
            market_cap_table.collection = metadatas.mint_address
            AND metadatas.address = metadata_jsons.metadata_address
    -- $1: addresses::text[]
    -- $2: start date::timestamp
    -- $3: end date::timestamp
    -- $4: limit::integer
    -- $5: offset::integer",
        order_direction = order_direction
    )
}
