//! Query utilities for `bonding_changes` table.

use diesel::{
    serialize::ToSql,
    sql_query,
    sql_types::{Int4, Text, Timestamp},
};

use crate::{
    db::{models::EnrichedBondingChange, Connection},
    prelude::*,
};

const CHANGES_QUERY: &str = r"
select * from (
  select
    address,
    slot,
    insert_ts,
    current_reserves_from_bonding - lag(current_reserves_from_bonding, 1) over (partition by address order by slot desc) reserve_change,
    current_supply_from_bonding - lag(current_supply_from_bonding, 1) over (partition by address order by slot desc) supply_change
  from bonding_changes
  where address = $1 and
        insert_ts >= $2 and insert_ts < $3
) s
where supply_change is not null and reserve_change <> 0
order by insert_ts desc
limit $4 offset $5;
 -- $1: address::text
 -- $2: start_ts::timestamp
 -- $3: stop_ts::timestamp
 -- $4: limit::integer
 -- $5: offset::integer
 ";

/// Return changes to the bonding supply and reserves over the time interval
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn list(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    start_ts: impl ToSql<Timestamp, Pg>,
    stop_ts: impl ToSql<Timestamp, Pg>,
    limit: impl ToSql<Int4, Pg>,
    offset: impl ToSql<Int4, Pg>,
) -> Result<Vec<EnrichedBondingChange>> {
    sql_query(CHANGES_QUERY)
        .bind(address)
        .bind(start_ts)
        .bind(stop_ts)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load enriched bonding changes")
}
