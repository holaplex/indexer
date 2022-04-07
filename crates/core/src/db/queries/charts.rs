use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text, Timestamp},
};

use crate::{
    db::{models::PricePoint, Connection},
    error::Result,
    prelude::*,
};

const FLOOR_PRICES_QUERY: &str = r"
select
    date_trunc('day', listed_at) as date
    min(listing_price) filter (where listing_canceled_at is null and listing_purchase_receipt is null)::bigint as price,

from (select lr.auction_house as auction_house,
        lr.price as listing_price, pr.price as purchase_price,
        pr.created_at as purchased_at,
        lr.created_at as listed_at,
        lr.purchase_receipt as listing_purchase_receipt,
        lr.canceled_at as listing_canceled_at,
        ah.treasury_mint as mint
from listing_receipts lr
    inner join auction_houses ah
        on (lr.auction_house = ah.address)
    left join purchase_receipts pr
        on (lr.purchase_receipt = pr.address)

where lr.auction_house = ANY($1) and lr.created_at >= $2 and lr.created_at <= $3
) as auction_house_stats
group by 1
order by 1 asc;
 -- $1: auction house addresses::text[]
 -- $2: start date::timestamp
 -- $3: end date::timestamp";

/// Load floor prices during a given date range for the given auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn floor_prices(
    conn: &Connection,
    auction_houses: impl ToSql<Text, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<PricePoint>> {
    diesel::sql_query(FLOOR_PRICES_QUERY)
        .bind(auction_houses)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load floor price chart")
}

const AVERAGE_PRICES_QUERY: &str = r"
select
    date_trunc('day', listed_at) as date
    round(avg(purchase_price))::bigint as price

from (select lr.auction_house as auction_house,
        lr.price as listing_price, pr.price as purchase_price,
        pr.created_at as purchased_at,
        lr.created_at as listed_at,
        lr.purchase_receipt as listing_purchase_receipt,
        lr.canceled_at as listing_canceled_at,
        ah.treasury_mint as mint
from listing_receipts lr
    inner join auction_houses ah
        on (lr.auction_house = ah.address)
    left join purchase_receipts pr
        on (lr.purchase_receipt = pr.address)

where lr.auction_house = ANY($1) and lr.created_at >= $2 and lr.created_at <= $3
) as auction_house_stats
group by 1
order by 1 asc;
 -- $1: auction house addresses::text[]
 -- $2: start date::timestamp
 -- $3: end date::timestamp";

/// Load average prices during a given date range for the given auction house address
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn average_prices(
    conn: &Connection,
    auction_houses: impl ToSql<Text, Pg>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<PricePoint>> {
    diesel::sql_query(AVERAGE_PRICES_QUERY)
        .bind(auction_houses)
        .bind::<Timestamp, _>(start_date)
        .bind::<Timestamp, _>(end_date)
        .load(conn)
        .context("Failed to load average price chart")
}
