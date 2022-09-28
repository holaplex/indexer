//! Query utilities for featured listings.

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Integer, Nullable, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::Listing, Connection},
    error::Result,
};

const FEATURED_LISTINGS_QUERY: &str = r"
SELECT
    a.id,
    a.trade_state,
    a.auction_house,
    a.marketplace_program,
    a.seller,
    a.metadata,
    a.purchase_id,
    a.price,
    a.token_size,
    a.trade_state_bump,
    a.created_at,
    a.canceled_at,
    a.slot,
    a.write_version,
    a.expiry

FROM (

    SELECT
        listings.*,
        row_number() OVER (
            PARTITION BY listings.seller ORDER BY listings.price DESC
        ) as row

    FROM listings, metadata_creators, wallet_totals

    WHERE
        metadata_creators.creator_address = wallet_totals.address
        AND listings.metadata = metadata_creators.metadata_address
        AND metadata_creators.share > 0
        AND metadata_creators.verified = true
        AND listings.purchase_id IS NULL
        AND listings.canceled_at IS NULL
        AND listings.auction_house = ANY($1)
        AND listings.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
        AND listings.created_at > current_date - interval '3 day'
        AND (($2 IS NULL) OR NOT(listings.seller = ANY($2)))

    GROUP BY listings.metadata, listings.id
    ORDER BY sum(wallet_totals.followers) DESC, listings.price DESC

) as a

WHERE a.row <= $3

LIMIT $4
OFFSET $5;
-- $1: auction_houses::text[]
-- $2: seller_exclusions::text[]
-- $3: limit_per_seller::integer
-- $4: limit::integer
-- $5: offset::integer";

/// Load featured listings
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn list(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    seller_exclusions: impl ToSql<Nullable<Array<Text>>, Pg>,
    limit_per_seller: impl ToSql<Integer, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Listing>> {
    diesel::sql_query(FEATURED_LISTINGS_QUERY)
        .bind(auction_houses)
        .bind(seller_exclusions)
        .bind(limit_per_seller)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load featured listings")
}
