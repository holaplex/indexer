//! Query utilities for using the store denylist table.

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
    db::{
        tables::{store_denylist, storefronts},
        Connection,
    },
    error::Result,
};

// Would you believe it took me 2 hours to debug type errors for this module?

type EqOwnerAddr<A> = Eq<store_denylist::owner_address, AsExprOf<A, VarChar>>;

/// The resulting type of the [`owner_address_ok`] function
pub type OwnerAddressOk<A> =
    Not<Exists<Filter<SelectStatement<store_denylist::table>, EqOwnerAddr<A>>>>;

/// Generate an SQL boolean expression that is true if `address` does not appear
/// in the `owner_address` column of the `store_denylist` table.
pub fn owner_address_ok<A: AsExpression<VarChar>>(address: A) -> OwnerAddressOk<A>
where
    SelectStatement<store_denylist::table>: FilterDsl<EqOwnerAddr<A>>,
    Filter<store_denylist::table, EqOwnerAddr<A>>: SelectQuery,
{
    not(exists(FilterDsl::filter(
        store_denylist::table,
        store_denylist::owner_address.eq(address),
    )))
}

/// Query all storefronts whose owner address is not in the denylist
#[must_use]
pub fn get_storefronts() -> Filter<storefronts::table, OwnerAddressOk<storefronts::owner_address>> {
    FilterDsl::filter(
        storefronts::table,
        owner_address_ok(storefronts::owner_address),
    )
}

/// Return entries in the store denylist that have been marked as hard-banned
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get_hard_banned<T: Queryable<Text, Pg>>(conn: &Connection) -> Result<Vec<T>> {
    FilterDsl::filter(store_denylist::table, store_denylist::hard_ban)
        .select(store_denylist::owner_address)
        .load(conn)
        .context("Query for hard-ban list failed")
}
