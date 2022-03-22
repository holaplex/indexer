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
    db::{
        models::{MarketStats, MintStats},
        Connection,
    },
    error::Result,
};

const MINT_QUERY: &str = r"
select
    ah.address                      as auction_house,
    ah.treasury_mint                as mint,
    min(f.min)::bigint              as floor,
    round(avg(pr.price))::bigint    as average,

    sum(case
        when ($2 - pr.created_at) < interval '24 hr'
            then pr.price
        else 0
    end)::bigint as volume_24hr

from auction_houses ah
    inner join purchase_receipts pr
        on (pr.auction_house = ah.address)

    inner join (
        select lr.auction_house, min(lr.price)::bigint
        from listing_receipts lr
        where lr.auction_house = any($1)
        group by lr.auction_house
    ) f on (f.auction_house = ah.address)

where ah.address = any($1)
group by ah.address;
 -- $1: auction house addresses::text[]
 -- $2: now::timestamp
";

/// Load per-mint statistics for the given auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn mint(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<MintStats>> {
    diesel::sql_query(MINT_QUERY)
        .bind(auction_houses)
        .bind::<Timestamp, _>(Local::now().naive_utc())
        .load(conn)
        .context("Failed to load mint stats")
}

const MARKET_QUERY: &str = r"
select
    sc.store_config_address                     as store_config,
    count(distinct mc.metadata_address)::bigint as nfts

from store_creators sc
    inner join metadata_creators mc
        on (mc.creator_address = sc.creator_address)

where sc.store_config_address = any($1) and mc.verified
group by sc.store_config_address;
 -- $1: store config addresses::text[]
";

/// Count the number of items in a marketplace
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn marketplace(
    conn: &Connection,
    store_configs: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<MarketStats>> {
    diesel::sql_query(MARKET_QUERY)
        .bind(store_configs)
        .load(conn)
        .context("Failed to load marketplace stats")
}
