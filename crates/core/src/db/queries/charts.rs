//! Query utilities for nft price charts

use anyhow::Context;
use chrono::NaiveDateTime;
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text, Timestamp},
};

use crate::{
    db::{models::PricePoint, Connection},
    error::Result,
};

const FLOOR_PRICES_QUERY: &str = r"
select series as date,
       coalesce(min(price), 0)::bigint as price
from generate_series($3::date, $4::date, '1 day'::interval) as series
left join (
    select date_trunc('day', created_at) as created_at_day, price
    from listing_receipts lr
    inner join metadatas md
    on lr.metadata = md.address
    inner join metadata_creators mc
    on md.address = mc.metadata_address
    where lr.auction_house = ANY($1) and mc.creator_address = ANY($2) and lr.created_at >= $3 and lr.created_at <= $4 and lr.canceled_at is null and lr.purchase_receipt is null
) as i
on i.created_at_day = series
group by date
order by date asc;
 -- $1: auction house addresses::text[]
 -- $2: creators addresses::text[]
 -- $3: start date::timestamp
 -- $4: end date::timestamp";

/// Load floor prices during a given date range for the desired auction house address per day
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn floor_prices(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    creators: impl ToSql<Array<Text>, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<PricePoint>> {
    diesel::sql_query(FLOOR_PRICES_QUERY)
        .bind(auction_houses)
        .bind(creators)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load floor prices")
}

const AVERAGE_PRICES_QUERY: &str = r"
select series as date,
       coalesce(round(avg(price)), 0)::bigint as price
from generate_series($3::date, $4::date, '1 day'::interval) as series
left join (
    select date_trunc('day', created_at) as created_at_day, price
    from purchase_receipts pr
    inner join metadatas md
    on pr.metadata = md.address
    inner join metadata_creators mc
    on md.address = mc.metadata_address
        where pr.auction_house = ANY($1) and mc.creator_address = ANY($2) and pr.created_at >= $3 and pr.created_at <= $4
) as i
on i.created_at_day = series
group by date
order by date asc;
 -- $1: auction house addresses::text[]
 -- $2: creators addresses::text[]
 -- $3: start date::timestamp
 -- $4: end date::timestamp";

/// Load average prices during a given date range for the desired auction house address per day
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn average_prices(
    conn: &Connection,
    creators: impl ToSql<Array<Text>, Pg>,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<PricePoint>> {
    diesel::sql_query(AVERAGE_PRICES_QUERY)
        .bind(auction_houses)
        .bind(creators)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load average prices")
}

const TOTAL_VOLUME_QUERY: &str = r"
select series as date,
       coalesce(round(sum(price)), 0)::bigint as price
from generate_series($3::date, $4::date, '1 day'::interval) as series
left join (
    select date_trunc('day', created_at) as created_at_day, price
    from purchase_receipts pr
    inner join metadatas md
    on pr.metadata = md.address
    inner join metadata_creators mc
    on md.address = mc.metadata_address
        where pr.auction_house = ANY($1) and mc.creator_address = ANY($2) and pr.created_at >= $3 and pr.created_at <= $4
) as i
on i.created_at_day = series
group by date
order by date asc;
 -- $1: auction house addresses::text[]
 -- $2: creators addresses::text[]
 -- $3: start date::timestamp
 -- $4: end date::timestamp";

/// Load total sales volum during a given date range for the desired auction house address per day
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn total_volume_prices(
    conn: &Connection,
    auction_houses: impl ToSql<Array<Text>, Pg>,
    creators: impl ToSql<Array<Text>, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<PricePoint>> {
    diesel::sql_query(TOTAL_VOLUME_QUERY)
        .bind(auction_houses)
        .bind(creators)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load average prices")
}
