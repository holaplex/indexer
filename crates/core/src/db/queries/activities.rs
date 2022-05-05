//! Query utilities for NFT activity.

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::NftActivity, Connection},
    error::Result,
};

// TODO: Add indexes on purchase_receipts for seller and buyer for joins
const ACTIVITES_QUERY: &str = r"
    SELECT listing_receipts.address as address, metadata, auction_house, price, created_at,
    array[seller] as wallets,
    array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
    'listing' as activity_type
        FROM listing_receipts
        LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listing_receipts.seller)
        WHERE auction_house = ANY($1)
    UNION
    SELECT purchase_receipts.address as address, metadata, auction_house, price, created_at,
    array[seller, buyer] as wallets,
    array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
    'purchase' as activity_type
        FROM purchase_receipts
        LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchase_receipts.seller)
        LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchase_receipts.buyer)
        WHERE auction_house = ANY($1)
    ORDER BY created_at DESC;
 -- $1: auction_houses::text[]";

/// Load all activities for desired auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn list(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<NftActivity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(auction_houses)
        .load(conn)
        .context("Failed to load activities")
}
