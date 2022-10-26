//! Query utilities for ``auction_house``

use diesel::{
    pg::Pg,
    sql_types::{Text, Timestamp},
    types::ToSql,
};

use crate::{
    db::{models::AuctionHouseVolume, Connection},
    error::Result,
    prelude::*,
};

const VOLUME_QUERY: &str = r"
SELECT COALESCE(SUM(purchase_price), 0) as volume
    FROM puchases p
    INNER JOIN auction_houses ah
        on (p.auction_house = ah.address)
    WHERE p.auction_house = $1
    AND p.created_at >= $2
    AND p.created_at <= $3
;

-- $1: address::text
-- $2: start_date::timestamp
-- $3: end_date::timestamp";

/// Load token distributed for reward center.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn volume(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<AuctionHouseVolume>> {
    diesel::sql_query(VOLUME_QUERY)
        .bind(address)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load volume for auction house")
}
