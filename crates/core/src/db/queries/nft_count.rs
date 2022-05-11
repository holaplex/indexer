//! Query utilities for looking up  nft counts

use diesel::{
    expression::{operators::Eq, AsExpression, NonAggregate},
    pg::Pg,
    prelude::*,
    query_builder::QueryFragment,
    query_source::joins::{Inner, Join, JoinOn},
    serialize::ToSql,
    sql_types::{Array, Text},
    AppearsOnTable,
};

use crate::{
    db::{
        any,
        models::StoreCreatorCount,
        tables::{
            bid_receipts, current_metadata_owners, listing_receipts, metadata_creators, metadatas,
        },
        Connection,
    },
    error::prelude::*,
};

/// Handles queries for total nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn total<C: ToSql<Text, Pg>>(conn: &Connection, creators: &[C]) -> Result<i64> {
    metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .filter(metadata_creators::creator_address.eq(any(creators)))
        .filter(metadata_creators::verified.eq(true))
        .count()
        .get_result(conn)
        .context("failed to load total nfts count")
}

/// Handles queries for listed nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn listed<C: ToSql<Text, Pg>, L: ToSql<Text, Pg>>(
    conn: &Connection,
    creators: &[C],
    listed: Option<&[L]>,
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
        .get_result(conn)
        .context("failed to load listed nfts count")
}

/// Handles queries for owned nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn owned<W: AsExpression<Text>, C: ToSql<Text, Pg>>(
    conn: &Connection,
    wallet: W,
    creators: Option<&[C]>,
) -> Result<i64>
where
    W::Expression: NonAggregate
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<metadatas::table, metadata_creators::table, Inner>,
                        Eq<metadatas::address, metadata_creators::metadata_address>,
                    >,
                    current_metadata_owners::table,
                    Inner,
                >,
                Eq<metadatas::mint_address, current_metadata_owners::mint_address>,
            >,
        >,
{
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(
            current_metadata_owners::table
                .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
        )
        .into_boxed();

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    query
        .filter(metadata_creators::verified.eq(true))
        .filter(current_metadata_owners::owner_address.eq(wallet))
        .count()
        .get_result(conn)
        .context("failed to load owned nfts count")
}

/// Handles queries for created nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn created<W: AsExpression<Text>>(conn: &Connection, wallet: W) -> Result<i64>
where
    W::Expression: NonAggregate
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<metadatas::table, metadata_creators::table, Inner>,
                Eq<metadatas::address, metadata_creators::metadata_address>,
            >,
        >,
{
    let query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .into_boxed();

    query
        .filter(metadata_creators::verified.eq(true))
        .filter(metadata_creators::creator_address.eq(wallet))
        .count()
        .get_result(conn)
        .context("failed to load created nfts count")
}

/// Handles queries for nfts count for a wallet with optional creators and auction house filters
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn offered<W: AsExpression<Text>, C: ToSql<Text, Pg>, H: ToSql<Text, Pg>>(
    conn: &Connection,
    wallet: W,
    creators: Option<&[C]>,
    auction_houses: Option<&[H]>,
) -> Result<i64>
where
    W::Expression: NonAggregate
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<metadatas::table, metadata_creators::table, Inner>,
                        Eq<metadatas::address, metadata_creators::metadata_address>,
                    >,
                    bid_receipts::table,
                    Inner,
                >,
                Eq<metadatas::address, bid_receipts::metadata>,
            >,
        >,
{
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
        .get_result(conn)
        .context("failed to load nfts count of open offers for a wallet")
}

/// Handles queries for wallet listed nfts count
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn wallet_listed<W: AsExpression<Text>, C: ToSql<Text, Pg>, L: ToSql<Text, Pg>>(
    conn: &Connection,
    wallet: W,
    creators: Option<&[C]>,
    listed: Option<&[L]>,
) -> Result<i64>
where
    W::Expression: NonAggregate
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<metadatas::table, metadata_creators::table, Inner>,
                        Eq<metadatas::address, metadata_creators::metadata_address>,
                    >,
                    listing_receipts::table,
                    Inner,
                >,
                Eq<metadatas::address, listing_receipts::metadata>,
            >,
        >,
{
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
        .get_result(conn)
        .context("failed to load listed nfts count")
}

const STORE_CREATOR_QUERY: &str = r"
select
    sc.creator_address as store_creator,
    count(distinct mc.metadata_address)::bigint as nfts

from store_creators sc
    inner join metadata_creators mc
        on (mc.creator_address = sc.creator_address)

where sc.creator_address = any($1) AND mc.verified
group by sc.creator_address;
 -- $1: store creator addresses::text[]";

/// Count the number of nfts created by a creator
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn store_creators(
    conn: &Connection,
    store_creators: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<StoreCreatorCount>> {
    diesel::sql_query(STORE_CREATOR_QUERY)
        .bind(store_creators)
        .load(conn)
        .context("Failed to load store creators counts")
}
