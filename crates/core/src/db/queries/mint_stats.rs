//! Retrieve per-mint statistics for an auction house.

use anyhow::Context;
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text},
};

use crate::{
    db::{models::MintStats, Connection},
    error::Result,
};

const QUERY: &str = r"
select
    ah.address as auction_house,
    ''::text as mint,
    0::bigint as floor,
    0::bigint as average,
    0::bigint as volume_24hr,
    0::bigint as count
from auction_houses ah
where ah.address = any($1);
";

/// Load per-mint statistics for the given auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn load(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<MintStats>> {
    diesel::sql_query(QUERY)
        .bind(auction_houses)
        .load(conn)
        .context("Failed to load mint stats")
}
