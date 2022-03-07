//! Query utilities for using the listing denylist table.

use anyhow::Context;
use diesel::{
    dsl::{exists, not, AsExprOf, Filter},
    expression::{exists::Exists, operators::Eq, AsExpression},
    helper_types::not as Not,
    pg::Pg,
    prelude::*,
    query_builder::{SelectQuery, SelectStatement},
    query_dsl::methods::FilterDsl,
    sql_types::{Text, VarChar},
};

use crate::{
    db::{tables::listing_denylist, Connection},
    error::Result,
};

type EqListingAddr<A> = Eq<listing_denylist::listing_address, AsExprOf<A, VarChar>>;

/// The resulting type of the [`listing_address_ok`] function
pub type OwnerAddressOk<A> =
    Not<Exists<Filter<SelectStatement<listing_denylist::table>, EqListingAddr<A>>>>;

/// Generate an SQL boolean expression that is true if `address` does not appear
/// in the `listing_address` column of the `listing_denylist` table.
pub fn listing_address_ok<A: AsExpression<VarChar>>(address: A) -> OwnerAddressOk<A>
where
    SelectStatement<listing_denylist::table>: FilterDsl<EqListingAddr<A>>,
    Filter<listing_denylist::table, EqListingAddr<A>>: SelectQuery,
{
    not(exists(FilterDsl::filter(
        listing_denylist::table,
        listing_denylist::listing_address.eq(address),
    )))
}

/// Return entries in the listing denylist that have been marked as hard-banned
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get_hard_banned<T: Queryable<Text, Pg>>(conn: &Connection) -> Result<Vec<T>> {
    FilterDsl::filter(listing_denylist::table, listing_denylist::hard_ban)
        .select(listing_denylist::listing_address)
        .load(conn)
        .context("Query for hard-ban list failed")
}
