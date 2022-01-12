//! Query utilities for using the store denylist table.

use anyhow::{bail, Context};
use diesel::{
    dsl::{exists, not, AsExprOf, Filter},
    expression::{exists::Exists, operators::Eq, AsExpression},
    helper_types::not as Not,
    prelude::*,
    query_builder::{SelectQuery, SelectStatement},
    query_dsl::methods::FilterDsl,
    sql_types::VarChar,
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

/// Query the store denylist for whether a store owner has been hard-banned
///
/// # Errors
/// This function fails if the underlying query fails to execute or returns too
/// many rows.
pub fn hard_banned(conn: &Connection, address: &str) -> Result<bool> {
    let mut it = FilterDsl::filter(
        store_denylist::table,
        store_denylist::owner_address.eq(address),
    )
    .select(store_denylist::hard_ban)
    .load::<bool>(conn)
    .context("Query for hard-ban failed")?
    .into_iter();

    let ret = it.next().unwrap_or(false);

    if it.next().is_some() {
        bail!("Invalid owner address for hard-ban query");
    }

    Ok(ret)
}
