//! Query utilities for collections.

use anyhow::Context;
use chrono::{DateTime, Utc};
use diesel::{
    expression::{operators::Eq, AsExpression, NonAggregate},
    pg::Pg,
    prelude::*,
    query_builder::{QueryFragment, QueryId},
    query_source::joins::{Inner, Join, JoinOn},
    serialize::ToSql,
    sql_types::{Array, Integer, Nullable, Text, Timestamp},
};

use crate::{
    db::{
        custom_types::OrderDirection,
        models::{Nft, NftActivity},
        queries::metadatas::NFT_COLUMNS,
        tables::{current_metadata_owners, metadata_collection_keys, metadata_jsons, metadatas},
        Connection,
    },
    error::Result,
    prelude::*,
};

/// Query collection by address
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn get<A: AsExpression<Text>>(conn: &Connection, address: A) -> Result<Option<Nft>>
where
    <A as AsExpression<Text>>::Expression: QueryId
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<
                            JoinOn<
                                Join<metadatas::table, metadata_jsons::table, Inner>,
                                Eq<
                                    metadatas::columns::address,
                                    metadata_jsons::columns::metadata_address,
                                >,
                            >,
                            metadata_collection_keys::table,
                            Inner,
                        >,
                        Eq<
                            metadata_collection_keys::columns::collection_address,
                            metadatas::columns::mint_address,
                        >,
                    >,
                    current_metadata_owners::table,
                    Inner,
                >,
                Eq<
                    current_metadata_owners::columns::mint_address,
                    metadatas::columns::mint_address,
                >,
            >,
        > + NonAggregate,
{
    metadatas::table
        .inner_join(
            metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
        )
        .inner_join(
            metadata_collection_keys::table
                .on(metadata_collection_keys::collection_address.eq(metadatas::mint_address)),
        )
        .inner_join(
            current_metadata_owners::table
                .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
        )
        .filter(metadata_collection_keys::collection_address.eq(address))
        .filter(metadata_collection_keys::verified.eq(true))
        .select(NFT_COLUMNS)
        .first::<Nft>(conn)
        .optional()
        .context("Failed to load Collection NFT by collection address")
}

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
        WITH collection_volumes AS (
            SELECT SUM(purchases.price) as total_volume,
            metadata_collection_keys.collection_address as collection
            FROM purchases
            INNER JOIN metadatas ON (metadatas.address = purchases.metadata)
            INNER JOIN metadata_collection_keys ON (metadata_collection_keys.metadata_address = metadatas.address)
            WHERE
            ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
            AND purchases.created_at >= $2
            AND purchases.created_at <= $3
            AND purchases.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY collection
            ORDER BY total_volume {order_direction}
            LIMIT $4
            OFFSET $5
        )         SELECT
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
                INNER JOIN collection_volumes ON (collection_volumes.collection = metadatas.mint_address)
                INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address);
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
        WITH market_caps AS (
            SELECT MIN(listings.price) * collection_stats.nft_count as market_cap,
            collection_stats.collection_address as collection
            FROM listings
            INNER JOIN metadatas ON (metadatas.address = listings.metadata)
            INNER JOIN metadata_collection_keys ON (metadata_collection_keys.metadata_address = metadatas.address)
            INNER JOIN collection_stats ON (collection_stats.collection_address = metadata_collection_keys.collection_address)
            WHERE listings.purchase_id IS NULL
            AND ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
            AND listings.canceled_at IS NULL
            AND listings.created_at >= $2
            AND listings.created_at <= $3
            AND listings.marketplace_program = 'M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K'
            GROUP BY collection_stats.collection_address
            ORDER BY market_cap {order_direction}
            LIMIT $4
            OFFSET $5
        )   SELECT
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
                INNER JOIN market_caps ON (market_caps.collection = metadatas.mint_address)
                INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address);
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
 -- $2: event_types::text[]
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
