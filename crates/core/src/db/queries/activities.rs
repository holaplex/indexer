use diesel::{
    pg::Pg,
    prelude::*,
    sql_types::{Array, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::Activity, Connection},
    error::Result,
    prelude::*,
};

const ACTIVITES_QUERY: &str = r"
    SELECT act.address, act.metadata, act.auction_house, act.price,
    act.created_at, act.wallets, act.activity_type, coalesce(metadatas.name, '') as metadata_name,
    coalesce(metadata_jsons.image, '') as metadata_image
    FROM
    (
    SELECT address, metadata, auction_house, price, created_at, array[seller::text] as wallets, 'listing' as activity_type
    FROM listing_receipts WHERE auction_house = ANY($1)
    UNION
    SELECT address, metadata, auction_house, price, created_at, array[seller::text, buyer::text] as wallets, 'purchase' as activity_type
        FROM purchase_receipts WHERE auction_house = ANY($1)
    ) as act
    LEFT JOIN metadatas ON act.metadata = metadatas.address
    LEFT JOIN metadata_jsons ON act.metadata = metadata_jsons.metadata_address
    ORDER BY act.created_at DESC;
 -- $1: auction_houses::text[]";

/// Load all activities for desired auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn activities(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<Activity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(auction_houses)
        .load(conn)
        .context("Failed to load activities")
}
