//! Query utilities for global stats

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::{
    db::{
        tables::{feed_events, mint_events, purchase_events},
        Connection,
    },
    error::prelude::*,
};

/// Return date of ith most recent mint
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn nth_mint_date(conn: &Connection, i: Option<u32>) -> Result<Vec<NaiveDateTime>> {
    let offset: u32 = i.unwrap_or(1000);

    feed_events::table
        .inner_join(mint_events::table.on(feed_events::id.eq(mint_events::feed_event_id)))
        .select(feed_events::created_at)
        .order(feed_events::created_at.desc())
        .limit(1)
        .offset(offset.try_into()?)
        .load(conn)
        .context("Failed to load recent mint")
}

/// Return date of ith most recent purchase
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn nth_purchase_date(conn: &Connection, i: Option<u32>) -> Result<Vec<NaiveDateTime>> {
    let offset: u32 = i.unwrap_or(1000);

    feed_events::table
        .inner_join(purchase_events::table.on(feed_events::id.eq(purchase_events::feed_event_id)))
        .select(feed_events::created_at)
        .order(feed_events::created_at.desc())
        .limit(1)
        .offset(offset.try_into()?)
        .load(conn)
        .context("Failed to load recent mint")
}
