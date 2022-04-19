//! Query utilities for `bonding_changes` table.

use diesel::{
    pg::Pg,
    serialize::ToSql,
    sql_query,
    sql_types::{Int4, Text, Timestamp},
};

use crate::{
    db::{models::EnrichedBondingChange, Connection},
    error::Result,
    prelude::*,
};

const CHANGES_QUERY: &str = r"
SELECT * FROM (
  SELECT
   address,
   slot,
   insert_ts,
   current_reserves_from_bonding - LAG(current_reserves_from_bonding, 1) OVER (PARTITION BY address ORDER BY slot DESC) reserve_change,
   current_supply_from_bonding - LAG(current_supply_from_bonding, 1) OVER (PARTITION BY address ORDER BY slot DESC) supply_change
  FROM bonding_changes
  WHERE address = $1 AND
        insert_ts >= $2 AND insert_ts < $3
) s
WHERE supply_change IS NOT NULL AND reserve_change <> 0
ORDER BY insert_ts DESC
LIMIT $4 OFFSET $5;
 -- $1: address::text
 -- $2: start_ts::datetime
 -- $3: stop_ts::datetime
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
        .context("failed to load enriched bonding changes")
}
