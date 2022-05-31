//! Query utilities for featured listings.

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Integer, Nullable, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::ListingReceipt, Connection},
    error::Result,
};

const FEATURED_LISTINGS_QUERY: &str = r"
SELECT a.address, a.trade_state, a.bookkeeper, a.auction_house, a.seller, a.metadata, a.purchase_receipt, a.price, a.token_size, a.bump, a.trade_state_bump, a.created_at, a.canceled_at from (
    SELECT listing_receipts.*, row_number() OVER (PARTITION BY listing_receipts.seller ORDER BY listing_receipts.price DESC) as row
    FROM listing_receipts, metadata_creators, wallet_totals

    WHERE
        metadata_creators.creator_address = wallet_totals.address
        AND listing_receipts.metadata = metadata_creators.metadata_address
        AND metadata_creators.share > 0
        AND metadata_creators.verified = true
        AND listing_receipts.purchase_receipt IS NULL
        AND listing_receipts.canceled_at IS NULL
        AND listing_receipts.auction_house = ANY($1)
        AND (($2 IS NULL) OR NOT(listing_receipts.seller = ANY($2)))

    GROUP BY listing_receipts.metadata, listing_receipts.address
    ORDER BY sum(wallet_totals.followers) DESC, listing_receipts.price DESC

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
) -> Result<Vec<ListingReceipt>> {
    diesel::sql_query(FEATURED_LISTINGS_QUERY)
        .bind(auction_houses)
        .bind(seller_exclusions)
        .bind(limit_per_seller)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load featured listings")
}
