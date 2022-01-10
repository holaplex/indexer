//! Query utilities for performing a join on storefronts, listings, listing
//! metadatas, and metadatas.

use std::collections::hash_map::Entry;

use anyhow::Context;
use chrono::{Duration, NaiveDateTime};
use diesel::{
    dsl::not,
    expression::{nullable::Nullable, operators::Eq as SqlEq},
    helper_types::{not as Not, And, Eq, Gt, IsNotNull, IsNull, Lt, Or},
    prelude::*,
    query_builder::SelectStatement,
    query_dsl::methods::{FilterDsl, LoadQuery, OrderDsl, SelectDsl},
    query_source::joins::{Inner, Join, JoinOn},
};
use solana_sdk::native_token::sol_to_lamports;

use super::store_denylist::{owner_address_ok, OwnerAddressOk};
use crate::{
    db::{
        models::ListingsTripleJoinRow,
        tables::{listing_metadatas, listings, metadatas, storefronts},
        Connection,
    },
    error::Result,
    hash::HashMap,
};

type EndsBefore = And<IsNotNull<listings::ends_at>, Lt<listings::ends_at, NaiveDateTime>>;
type EndsAfter = And<IsNotNull<listings::ends_at>, Gt<listings::ends_at, NaiveDateTime>>;
type EndedInstantSale = And<IsNull<listings::ends_at>, IsNotNull<listings::highest_bid>>;
type PriceFloorGt = And<IsNotNull<listings::price_floor>, Gt<listings::price_floor, i64>>;

type And3<A, B, C> = And<And<A, B>, C>;
type Or4<A, B, C, D> = Or<Or<Or<A, B>, C>, D>;

/// The type of the listing rejection filter
pub type Rejected = And3<
    Eq<listings::ended, bool>,
    Not<Or4<EndsBefore, EndedInstantSale, EndsAfter, PriceFloorGt>>,
    OwnerAddressOk<storefronts::owner_address>,
>;

/// Filter an SQL query to reject listings we don't want to return
#[must_use]
pub fn rejected(now: NaiveDateTime) -> Rejected {
    let in_31_days = now + Duration::days(31);
    let too_expensive: i64 = sol_to_lamports(2000.0)
        .try_into()
        .unwrap_or_else(|_| unreachable!());

    let already_ended: EndsBefore = listings::ends_at
        .is_not_null()
        .and(listings::ends_at.lt(now));
    let ended_instant_sale: EndedInstantSale = listings::ends_at
        .is_null()
        .and(listings::highest_bid.is_not_null());
    let not_ending_soon: EndsAfter = listings::ends_at
        .is_not_null()
        .and(listings::ends_at.gt(in_31_days));
    let too_expensive: PriceFloorGt = listings::price_floor
        .is_not_null()
        .and(listings::price_floor.gt(too_expensive));

    listings::ended
        .eq(false)
        .and(not(already_ended
            .or(ended_instant_sale)
            .or(not_ending_soon)
            .or(too_expensive)))
        .and(owner_address_ok(storefronts::owner_address))
}

/// The type of the required column layout
pub type Columns = (
    listings::address,
    listings::ends_at,
    listings::created_at,
    listings::ended,
    listings::highest_bid,
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
pub const COLUMNS: Columns = (
    listings::address,
    listings::ends_at,
    listings::created_at,
    listings::ended,
    listings::highest_bid,
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

/// The type of the expected ORDER BY clause
pub type Order = (listings::address, listing_metadatas::metadata_index);

/// Expected ORDER BY clause for a listings triple-join
pub const ORDER: Order = (listings::address, listing_metadatas::metadata_index);

/// The resulting type of joining the four tables
pub type TripleJoin = SelectStatement<
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
                SqlEq<Nullable<listing_metadatas::listing_address>, Nullable<listings::address>>,
            >,
            storefronts::table,
            Inner,
        >,
        SqlEq<Nullable<listings::store_owner>, Nullable<storefronts::owner_address>>,
    >,
>;

/// Join the four input tables
pub fn join() -> TripleJoin {
    listings::table
        .inner_join(listing_metadatas::table.inner_join(metadatas::table))
        .inner_join(storefronts::table)
}

/// The resulting type of the required WHERE clause
pub type Filter<T> = diesel::dsl::Filter<T, Rejected>;

/// Filter the given query to remove listings we don't want to return
pub fn filter<T: FilterDsl<Rejected>>(query: T, now: NaiveDateTime) -> Filter<T> {
    query.filter(rejected(now))
}

/// The resulting type of the required SELECT statement
pub type Select<T> = diesel::dsl::Select<T, Columns>;

/// Select the required columns from the input query
pub fn select<T: SelectDsl<Columns>>(from: T) -> Select<T> {
    from.select(COLUMNS)
}

/// The resulting type of the ORDER BY clause
pub type OrderBy<T> = diesel::dsl::Order<T, Order>;

/// Order the results by the expected columns
pub fn order<T: OrderDsl<Order>>(query: T) -> OrderBy<T> {
    query.order(ORDER)
}

fn group_rows<
    T: From<ListingsTripleJoinRow> + Extend<ListingsTripleJoinRow>,
    I: FromIterator<T>,
>(
    rows: Vec<ListingsTripleJoinRow>,
) -> I {
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

    listings.into_values().collect()
}

/// Load the given query on the listings triple join tables, group results
/// include every listing, this is intended for API use only. BE CAREFUL when
/// using this to power a UI, you can get listings with undesirable content
///
/// # Errors
/// This function fails if the underlying SQL query cannot successfully be executed
///
/// # Safety
/// This function will load listings from stores that have been banned, priced irrationally or very old
pub unsafe fn load_unfiltered<
    Q,
    T: From<ListingsTripleJoinRow> + Extend<ListingsTripleJoinRow>,
    O: FromIterator<T>,
>(
    query: impl FnOnce(OrderBy<Select<TripleJoin>>) -> Q,
    conn: &Connection,
) -> Result<O>
where
    Q: LoadQuery<Connection, ListingsTripleJoinRow>,
{
    let rows = query(order(select(join())))
        .load(conn)
        .context("Query for listings triple-join failed")?;

    Ok(group_rows(rows))
}

/// Load the given query on the listings triple join tables, and group the results.
/// Rejects undesirable listings by price, ended date and others, see [`reject`]
///
/// # Errors
/// This function fails if the underlying SQL query cannot successfully be executed
pub fn load<Q, T: From<ListingsTripleJoinRow> + Extend<ListingsTripleJoinRow>, O: FromIterator<T>>(
    query: impl FnOnce(OrderBy<Select<Filter<TripleJoin>>>) -> Q,
    conn: &Connection,
    now: NaiveDateTime,
) -> Result<O>
where
    Q: LoadQuery<Connection, ListingsTripleJoinRow>,
{
    let rows = query(order(select(filter(join(), now))))
        .load(conn)
        .context("Query for filtered listings triple-join failed")?;

    Ok(group_rows(rows))
}
