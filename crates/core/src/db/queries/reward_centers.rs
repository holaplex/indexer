//! Query utilities for `reward_centers` table.

use diesel::{
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::{QueryFragment, QueryId},
    sql_types::{Integer, Nullable, Text, Timestamp},
    types::ToSql,
    AppearsOnTable,
};

use crate::{
    db::{
        models::{ReadRewardPayout, RewardCenter, TokensDistributed},
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

const TOKENS_DISTRIBUTED_QUERY: &str = r"
SELECT COALESCE(SUM(reward_payouts.seller_reward + reward_payouts.seller_reward), 0) as tokens_distributed
    FROM reward_payouts
    WHERE reward_payouts.reward_center = $1
    AND ($2 is null or reward_payouts.created_at >= $2)
    AND ($3 is null or reward_payouts.created_at <= $3)
;

-- $1: address::text
-- $2: start_date::timestamp
-- $3: end_date::timestamp";

/// Load token distributed for reward center.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn tokens_distributed(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    start_date: Option<NaiveDateTime>,
    end_date: Option<NaiveDateTime>,
) -> Result<Vec<TokensDistributed>> {
    diesel::sql_query(TOKENS_DISTRIBUTED_QUERY)
        .bind(address)
        .bind::<Nullable<Timestamp>, _>(start_date)
        .bind::<Nullable<Timestamp>, _>(end_date)
        .load(conn)
        .context("Failed to load tokens distributed")
}
