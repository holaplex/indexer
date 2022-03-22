//! Retrieve per-mint statistics for an auction house.

use anyhow::Context;
use chrono::Local;
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text, Timestamp},
};

use crate::{
    db::{models::MintStats, Connection},
    error::Result,
};

const QUERY: &str = r"
select
    ah.address                      as auction_house,
    ah.treasury_mint                as mint,
    min(pr.price)::bigint           as floor,
    round(avg(pr.price))::bigint    as average,

    sum(case
        when ($2 - pr.created_at) < interval '24 hr'
            then pr.price
        else 0
    end)::bigint as volume_24hr,

    sum(c.count)::bigint as count

from auction_houses ah
    inner join purchase_receipts pr
        on (pr.auction_house = ah.address)

    inner join (
        select lr.auction_house, count(distinct lr.metadata)
        from listing_receipts lr
        group by lr.auction_house
    ) c on (c.auction_house = ah.address)

where ah.address = any($1)
group by ah.address;
-- $1: auction house addresses::text[]
-- $2: now::timestamp
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
        .bind::<Timestamp, _>(Local::now().naive_utc())
        .load(conn)
        .context("Failed to load mint stats")
}
