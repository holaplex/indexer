//! Query utilities for `reward_centers` table.

use diesel::{
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::{QueryFragment, QueryId},
    sql_types::Text,
    AppearsOnTable,
};

use crate::{
    db::{models::RewardCenter, tables::reward_centers, Connection},
    error::Result,
    prelude::*,
};

/// Return reward center by address
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get<A: AsExpression<Text>>(conn: &Connection, address: A) -> Result<RewardCenter>
where
    A::Expression: NonAggregate + QueryId + QueryFragment<Pg> + AppearsOnTable<reward_centers::table>,
{
    reward_centers::table
        .select(reward_centers::all_columns)
        .filter(reward_centers::address.eq(address))
        .first::<RewardCenter>(conn)
        .map_err(Into::into)
}
