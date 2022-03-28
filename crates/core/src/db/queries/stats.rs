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
    prelude::*,
};

const MINT_QUERY: &str = r"
select
    auction_house,
    mint,
    min(listing_price) filter (where token_account_amount = 1 and listing_canceled_at is null and listing_purchase_receipt is null) as floor,
    round(avg(purchase_price))::bigint as average,
    sum(purchase_price) filter (where ($3 - purchased_at) < interval '24 hr') as volume_24hr

from (select lr.auction_house as auction_house,
        mc.creator_address as creator_address,
        lr.price as listing_price, pr.price as purchase_price,
        pr.created_at as purchased_at,
        lr.created_at as listed_at,
        lr.purchase_receipt as listing_purchase_receipt,
        lr.canceled_at as listing_canceled_at,
        ta.amount as token_account_amount,
        ah.treasury_mint as mint
from listing_receipts lr
    inner join auction_houses ah
        on (lr.auction_house = ah.address)
    inner join metadatas md
        on (lr.metadata = md.address)
    inner join metadata_creators mc
        on (md.address = mc.metadata_address)
    inner join token_accounts ta
        on (md.mint_address = ta.mint_address)
    left join purchase_receipts pr
        on (lr.purchase_receipt = pr.address)

where lr.auction_house = ANY($1)
) as auction_house_stats
group by auction_house, mint;
 -- $1: auction house addresses::text[]
 -- $2: now::timestamp";

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
 -- $1: store config addresses::text[]";

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

const COLLECTION_QUERY: &str = r"
select
    auction_house,
    mint,
    min(listing_price) filter (where token_account_amount = 1 and listing_canceled_at is null and listing_purchase_receipt is null) as floor,
    round(avg(purchase_price))::bigint as average,
    sum(purchase_price) filter (where ($3 - purchased_at) < interval '24 hr') as volume_24hr

from (
    select lr.auction_house as auction_house,
        mc.creator_address as creator_address,
        lr.price as listing_price, pr.price as purchase_price,
        pr.created_at as purchased_at,
        lr.created_at as listed_at,
        lr.purchase_receipt as listing_purchase_receipt,
        lr.canceled_at as listing_canceled_at,
        ta.amount as token_account_amount,
        ah.treasury_mint as mint
from listing_receipts lr
    inner join auction_houses ah
        on (lr.auction_house = ah.address)
    inner join metadatas md
        on (lr.metadata = md.address)
    inner join metadata_creators mc
        on (md.address = mc.metadata_address)
    inner join token_accounts ta
        on (md.mint_address = ta.mint_address)
    left join purchase_receipts pr
        on (lr.purchase_receipt = pr.address)

where lr.auction_house = ANY($1)
    and mc.creator_address = $2
    and mc.verified
) as collection_stats
group by auction_house, mint;
 -- $1: auction house addresses::text[]
 -- $2: creator::text
 -- $3: now::timestamp";

/// Load per-mint statistics for the given creator for provided auction houses
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn collection(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    creator: impl ToSql<Text, Pg>,
) -> Result<Vec<MintStats>> {
    diesel::sql_query(COLLECTION_QUERY)
        .bind(auction_houses)
        .bind(creator)
        .bind::<Timestamp, _>(Local::now().naive_utc())
        .load(conn)
        .context("Failed to load collection mint stats")
}
