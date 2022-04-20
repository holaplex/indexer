use diesel::{
    pg::Pg,
    prelude::*,
    sql_types::{Array, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::NftActivity, Connection},
    error::Result,
    prelude::*,
};

const ACTIVITES_QUERY: &str = r"
    SELECT address, metadata, auction_house, price, created_at, array[seller::text] as wallets, 'listing' as activity_type
    FROM listing_receipts WHERE auction_house = ANY($1)
    UNION
    SELECT address, metadata, auction_house, price, created_at, array[seller::text, buyer::text] as wallets, 'purchase' as activity_type
        FROM purchase_receipts WHERE auction_house = ANY($1)
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
