//! Query utilities for NFT activity.

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Nullable, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::NftActivity, Connection},
    error::Result,
};

const ACTIVITES_QUERY: &str = r"
SELECT listings.id as id, metadata, auction_house, price, created_at, marketplace_program
    array[seller] as wallets,
    array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
    'listing' as activity_type
        FROM listings
        LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listings.seller)
        INNER JOIN metadatas md
        on listings.metadata = md.address
        INNER JOIN metadata_creators mc
        on md.address = mc.metadata_address
        WHERE auction_house = ANY($1) and ($2 is null OR mc.creator_address = ANY($2))
    UNION
    SELECT purchases.id as id, metadata, auction_house, price, created_at, marketplace_program,
    array[seller, buyer] as wallets,
    array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
    'purchase' as activity_type
        FROM purchases
        LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchases.seller)
        LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchases.buyer)
        INNER JOIN metadatas md
        on purchases.metadata = md.address
        INNER JOIN metadata_creators mc
        on md.address = mc.metadata_address
        WHERE auction_house = ANY($1) and ($2 is null OR mc.creator_address = ANY($2))
    ORDER BY created_at DESC;
 -- $1: auction_houses::text[]
 -- $2: creators::text[]";

/// Load all activities for desired auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn list(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    creators: impl ToSql<Nullable<Array<Text>>, Pg>,
) -> Result<Vec<NftActivity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(auction_houses)
        .bind(creators)
        .load(conn)
        .context("Failed to load activities")
}
