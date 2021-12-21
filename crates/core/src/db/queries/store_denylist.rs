//! Query utilities for using the store denylist table.

use diesel::{
    dsl::{exists, not, AsExprOf, Filter},
    expression::{exists::Exists, operators::Eq, AsExpression},
    helper_types::not as Not,
    prelude::*,
    query_builder::{SelectQuery, SelectStatement},
    query_dsl::methods::FilterDsl,
    sql_types::VarChar,
};

use crate::db::tables::{store_denylist, storefronts};

// Would you believe it took me 2 hours to debug type errors for this module?

type EqOwnerAddr<A> = Eq<store_denylist::owner_address, AsExprOf<A, VarChar>>;
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
pub fn get_storefronts() -> Filter<storefronts::table, OwnerAddressOk<storefronts::owner_address>> {
    FilterDsl::filter(
        storefronts::table,
        owner_address_ok(storefronts::owner_address),
    )
}
