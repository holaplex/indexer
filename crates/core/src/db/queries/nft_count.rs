//! Query utilities for looking up  nft counts

use diesel::prelude::*;

use crate::{
    db::{
        any,
        tables::{bid_receipts, listing_receipts, metadata_creators, metadatas, token_accounts},
        Connection,
    },
    error::prelude::*,
};

/// Handles queries for total nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn total(conn: &Connection, creators: Vec<String>) -> Result<i64> {
    metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .filter(metadata_creators::creator_address.eq(any(creators)))
        .filter(metadata_creators::verified.eq(true))
        .count()
        .get_result::<i64>(conn)
        .context("failed to load total nfts count")
}

/// Handles queries for listed nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn listed(
    conn: &Connection,
    creators: Vec<String>,
    listed: Option<Vec<String>>,
) -> Result<i64> {
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(listing_receipts::table.on(metadatas::address.eq(listing_receipts::metadata)))
        .into_boxed();

    if let Some(listed) = listed {
        query = query.filter(listing_receipts::auction_house.eq(any(listed)));
    }

    query
        .filter(metadata_creators::creator_address.eq(any(creators)))
        .filter(metadata_creators::verified.eq(true))
        .filter(listing_receipts::purchase_receipt.is_null())
        .filter(listing_receipts::canceled_at.is_null())
        .count()
        .get_result::<i64>(conn)
        .context("failed to load listed nfts count")
}

/// Handles queries for owned nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn owned(conn: &Connection, wallet: String, creators: Option<Vec<String>>) -> Result<i64> {
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(
            token_accounts::table.on(metadatas::mint_address.eq(token_accounts::mint_address)),
        )
        .into_boxed();

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    query
        .filter(metadata_creators::verified.eq(true))
        .filter(token_accounts::amount.eq(1))
        .filter(token_accounts::owner_address.eq(wallet))
        .count()
        .get_result::<i64>(conn)
        .context("failed to load owned nfts count")
}

/// Handles queries for nfts count for a wallet with optional creators and auction house filters
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn offered(
    conn: &Connection,
    wallet: String,
    creators: Option<Vec<String>>,
    auction_houses: Option<Vec<String>>,
) -> Result<i64> {
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(bid_receipts::table.on(metadatas::address.eq(bid_receipts::metadata)))
        .into_boxed();

    if let Some(auction_houses) = auction_houses {
        query = query.filter(bid_receipts::auction_house.eq(any(auction_houses)));
    }

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    query
        .filter(metadata_creators::verified.eq(true))
        .filter(bid_receipts::buyer.eq(wallet))
        .filter(bid_receipts::purchase_receipt.is_null())
        .filter(bid_receipts::canceled_at.is_null())
        .count()
        .get_result::<i64>(conn)
        .context("failed to load nfts count of open offers for a wallet")
}

/// Handles queries for wallet listed nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn wallet_listed(
    conn: &Connection,
    wallet: String,
    creators: Option<Vec<String>>,
    listed: Option<Vec<String>>,
) -> Result<i64> {
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(listing_receipts::table.on(metadatas::address.eq(listing_receipts::metadata)))
        .into_boxed();

    if let Some(listed) = listed {
        query = query.filter(listing_receipts::auction_house.eq(any(listed)));
    }

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    query
        .filter(metadata_creators::verified.eq(true))
        .filter(listing_receipts::purchase_receipt.is_null())
        .filter(listing_receipts::canceled_at.is_null())
        .filter(listing_receipts::seller.eq(wallet))
        .count()
        .get_result::<i64>(conn)
        .context("failed to load listed nfts count")
}
