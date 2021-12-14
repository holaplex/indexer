//! Reusable query operations for common or complicated queries.

use std::collections::hash_map::Entry;

use anyhow::Context;
use diesel::{
    expression::{nullable::Nullable, operators::Eq as SqlEq},
    prelude::*,
    query_builder::SelectStatement,
    query_source::joins::{Inner, Join, JoinOn},
};

use super::{
    models::ListingsTripleJoinRow,
    tables::{listing_metadatas, listings, metadatas, storefronts},
};
use crate::{
    error::{Error, Result},
    hash::HashMap,
};

type ListingsTripleJoinCols = (
    listings::address,
    listings::ends_at,
    listings::created_at,
    listings::ended,
    listings::last_bid,
    listings::last_bid_time,
    listings::price_floor,
    listings::total_uncancelled_bids,
    listings::instant_sale_price,
    storefronts::subdomain,
    storefronts::title,
    metadatas::address,
    metadatas::name,
    metadatas::uri,
);

/// Required column layout for a listings triple-join
pub const LISTINGS_TRIPLE_JOIN_COLS: ListingsTripleJoinCols = (
    listings::address,
    listings::ends_at,
    listings::created_at,
    listings::ended,
    listings::last_bid,
    listings::last_bid_time,
    listings::price_floor,
    listings::total_uncancelled_bids,
    listings::instant_sale_price,
    storefronts::subdomain,
    storefronts::title,
    metadatas::address,
    metadatas::name,
    metadatas::uri,
);

/// Expected ORDER BY clause for a listings triple-join
pub const LISTINGS_TRIPLE_JOIN_ORDER: (listings::address, listing_metadatas::metadata_index) =
    (listings::address, listing_metadatas::metadata_index);

/// Perform a query on the join of (`storefronts`, `listings`,
/// `listing_metadatas`, `metadatas`)
///
/// # Errors
/// This function fails if the underlying SQL query cannot successfully be executed
pub fn listings_triple_join<
    E: Into<Error>,
    T: From<ListingsTripleJoinRow> + Extend<ListingsTripleJoinRow>,
    O: FromIterator<T>,
>(
    run_query: impl FnOnce(
        SelectStatement<
            JoinOn<
                Join<
                    JoinOn<
                        Join<
                            listings::table,
                            SelectStatement<
                                JoinOn<
                                    Join<listing_metadatas::table, metadatas::table, Inner>,
                                    SqlEq<
                                        Nullable<listing_metadatas::metadata_address>,
                                        Nullable<metadatas::address>,
                                    >,
                                >,
                            >,
                            Inner,
                        >,
                        SqlEq<
                            Nullable<listing_metadatas::listing_address>,
                            Nullable<listings::address>,
                        >,
                    >,
                    storefronts::table,
                    Inner,
                >,
                SqlEq<Nullable<listings::store_owner>, Nullable<storefronts::owner_address>>,
            >,
        >,
    ) -> Result<Vec<ListingsTripleJoinRow>, E>,
) -> Result<O> {
    let rows = run_query(
        listings::table
            .inner_join(listing_metadatas::table.inner_join(metadatas::table))
            .inner_join(storefronts::table),
    )
    .map_err(Into::into)
    .context("Query for listings triple-join failed")?;
    let mut listings = HashMap::default();

    for row in rows {
        match listings.entry(row.address.clone()) {
            Entry::Vacant(v) => {
                v.insert(T::from(row));
            },
            Entry::Occupied(o) => {
                o.into_mut().extend(Some(row));
            },
        }
    }

    Ok(listings.into_values().collect())
}
