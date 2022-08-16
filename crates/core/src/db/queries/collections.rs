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
    db::{
        custom_types::OrderDirection,
        models::{Nft, NftActivity},
        Connection,
    },
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
                INNER JOIN auction_houses on (purchases.auction_house = auction_houses.address)
                WHERE
                    ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
                    AND auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
                    AND purchases.created_at >= $2
                    AND purchases.created_at <= $3
                    AND metadata_collection_keys.verified = true
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
            metadata_jsons.animation_url,
            metadata_jsons.external_url,
            metadata_jsons.category,
            metadata_jsons.model,
            current_metadata_owners.token_account_address
        FROM metadatas
        INNER JOIN metadata_jsons ON (metadata_jsons.metadata_address = metadatas.address)
        INNER JOIN volume_table ON (volume_table.collection = metadatas.mint_address)
        INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address)
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
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Nft>> {
    diesel::sql_query(make_by_market_cap_query_string(order_direction))
        .bind(addresses)
        .bind::<Timestamp, _>(start_date.naive_utc())
        .bind::<Timestamp, _>(end_date.naive_utc())
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load collections by market cap")
}

fn make_by_market_cap_query_string(order_direction: OrderDirection) -> String {
    format!(
        r"
        WITH market_cap_table (collection, market_cap) AS (
            SELECT floor_prices.collection, MIN(floor_prices.floor_price) * MAX(nft_counts.nft_count) AS market_cap
            FROM (
                    SELECT metadata_collection_keys.collection_address AS collection, MIN(listings.price) AS floor_price
                    FROM listings
                    INNER JOIN metadatas ON(listings.metadata = metadatas.address)
                    INNER JOIN metadata_collection_keys ON(metadatas.address = metadata_collection_keys.metadata_address)
                    INNER JOIN auction_houses ON(listings.auction_house = auction_houses.address)
                    WHERE
                        ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
                        AND auction_houses.treasury_mint = 'So11111111111111111111111111111111111111112'
                        AND listings.created_at >= $2
                        AND listings.created_at <= $3
                        AND listings.purchase_id IS NULL
                        AND listings.canceled_at IS NULL
                        AND metadata_collection_keys.verified = true
                    GROUP BY metadata_collection_keys.collection_address
                    HAVING COUNT(listings) > 2
                ) floor_prices,
                (
                    SELECT metadata_collection_keys.collection_address AS collection, COUNT (metadata_collection_keys.metadata_address) AS nft_count
                    FROM metadata_collection_keys
                    WHERE metadata_collection_keys.verified = true
                    GROUP BY metadata_collection_keys.collection_address
                ) nft_counts
                WHERE nft_counts.collection = floor_prices.collection
                GROUP BY floor_prices.collection
            ORDER BY market_cap {order_direction}
            LIMIT $4
            OFFSET $5
        )
        SELECT
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
            metadata_jsons.animation_url,
            metadata_jsons.external_url,
            metadata_jsons.category,
            metadata_jsons.model,
            current_metadata_owners.token_account_address
        FROM metadatas
        INNER JOIN metadata_jsons ON (metadata_jsons.metadata_address = metadatas.address)
        INNER JOIN market_cap_table ON (market_cap_table.collection = metadatas.mint_address)
        INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address)
    -- $1: addresses::text[]
    -- $2: start date::timestamp
    -- $3: end date::timestamp
    -- $4: limit::integer
    -- $5: offset::integer",
        order_direction = order_direction
    )
}

const COLLECTION_ACTIVITES_QUERY: &str = r"
SELECT listings.id as id, metadata, auction_house, price, created_at, marketplace_program,
    array[seller] as wallets,
    array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
    'listing' as activity_type
        FROM listings
        LEFT JOIN twitter_handle_name_services ON(twitter_handle_name_services.wallet_address = listings.seller)
        INNER JOIN metadata_collection_keys ON(metadata_collection_keys.metadata_address = listings.metadata)
        WHERE metadata_collection_keys.collection_address = $1
        AND ('LISTINGS' = ANY($2) OR $2 IS NULL)
    UNION
    SELECT purchases.id as id, metadata, auction_house, price, created_at, marketplace_program,
    array[seller, buyer] as wallets,
    array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
    'purchase' as activity_type
        FROM purchases
        LEFT JOIN twitter_handle_name_services sth ON(sth.wallet_address = purchases.seller)
        LEFT JOIN twitter_handle_name_services bth ON(bth.wallet_address = purchases.buyer)
        INNER JOIN metadata_collection_keys ON(metadata_collection_keys.metadata_address = purchases.metadata)
        WHERE metadata_collection_keys.collection_address = $1
        AND ('PURCHASES' = ANY($2) OR $2 IS NULL)
    UNION
    SELECT offers.id as id, metadata, auction_house, price, created_at, marketplace_program,
    array[buyer] as wallets,
    array[bth.twitter_handle] as wallet_twitter_handles,
    'offer' as activity_type
        FROM offers
        LEFT JOIN twitter_handle_name_services bth ON(bth.wallet_address = offers.buyer)
        INNER JOIN metadata_collection_keys ON(metadata_collection_keys.metadata_address = offers.metadata)
        WHERE metadata_collection_keys.collection_address = $1
        AND offers.purchase_id IS NULL
        AND ('OFFERS' = ANY($2) OR $2 IS NULL)
    ORDER BY created_at DESC
    LIMIT $3
    OFFSET $4;

 -- $1: address::text
 -- $2: evnet_types::text[]
 -- $3: limit::integer
 -- $4: offset::integer";

/// Load listing, sales, offers activity for a collection
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn collection_activities(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    event_types: impl ToSql<Nullable<Array<Text>>, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<NftActivity>> {
    diesel::sql_query(COLLECTION_ACTIVITES_QUERY)
        .bind(address)
        .bind(event_types)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load collection activities")
}
