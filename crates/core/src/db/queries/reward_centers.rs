//! Query utilities for `reward_centers` table.

use diesel::{
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::{QueryFragment, QueryId},
    sql_types::{Integer, Text},
    types::ToSql,
    AppearsOnTable,
};

use crate::{
    db::{
        models::{ReadRewardPayout, RewardCenter},
        tables::reward_centers,
        Connection,
    },
    error::Result,
    prelude::*,
};

/// Return reward center by address
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get<A: AsExpression<Text>>(conn: &Connection, address: A) -> Result<RewardCenter>
where
    A::Expression:
        NonAggregate + QueryId + QueryFragment<Pg> + AppearsOnTable<reward_centers::table>,
{
    reward_centers::table
        .select(reward_centers::all_columns)
        .filter(reward_centers::address.eq(address))
        .first::<RewardCenter>(conn)
        .map_err(Into::into)
}

const PAYOUTS_QUERY: &str = r"
SELECT purchase_id, metadata, reward_center, buyer, buyer_reward, seller, seller_reward,
created_at, reward_payouts.slot as slot, reward_payouts.write_version as write_version,
bth.twitter_handle as buyer_twitter_handle,
sth.twitter_handle as seller_twitter_handle
    FROM reward_payouts
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = reward_payouts.buyer)
    LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = reward_payouts.seller)
    WHERE reward_payouts.reward_center = $1
ORDER BY created_at DESC
LIMIT $2
OFFSET $3;

-- $1: address::text
-- $2: limit::integer
-- $3: offset::integer";

/// Load payouts for reward center.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn payouts(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<ReadRewardPayout>> {
    diesel::sql_query(PAYOUTS_QUERY)
        .bind(address)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load reward center payouts")
}
