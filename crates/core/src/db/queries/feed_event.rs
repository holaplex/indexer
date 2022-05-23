//! Query utilities for feed events.

use diesel::{
    dsl::not,
    expression::{nullable::Nullable, operators::Eq, AsExpression, NonAggregate},
    prelude::*,
    query_builder::{QueryFragment, QueryId},
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::Text,
    AppearsOnTable,
};

use crate::{
    db::{
        any, models,
        pg::Pg,
        tables::{
            feed_event_wallets, feed_events, follow_events, graph_connections, listing_events,
            mint_events, offer_events, purchase_events, twitter_handle_name_services,
        },
        Connection,
    },
    error::prelude::*,
};

/// join of event tables into a single event type enriched with twitter info for source wallet
pub type Columns<'a> = (
    models::FeedEvent,
    // source wallet address for the event
    String,
    // twitter handle of the source wallet if available
    Option<String>,
    Option<models::MintEvent<'a>>,
    Option<models::OfferEvent<'a>>,
    Option<models::ListingEvent<'a>>,
    Option<models::PurchaseEvent<'a>>,
    Option<models::FollowEvent<'a>>,
);

/// feed event types, to be used for filtering feed events
#[derive(Debug, Clone, Copy, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum EventType {
    /// Mint Events
    Mint,

    /// Offer Events
    Offer,

    /// Listing Events
    Listing,

    /// Purchase Events
    Purchase,

    /// Follow Events
    Follow,
}

/// Return polymorphic list of feed events based on who the wallet is following
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn list<W: Clone + AsExpression<Text>>(
    conn: &Connection,
    wallet: W,
    limit: i64,
    offset: i64,
    exclude_types: Option<Vec<EventType>>
) -> Result<
    Vec<Columns>,
> where <W as AsExpression<Text>>::Expression: NonAggregate + AppearsOnTable<Join<graph_connections::table, JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<feed_event_wallets::table, feed_events::table, Inner>, Eq<Nullable<feed_event_wallets::feed_event_id>, Nullable<feed_events::id>>>, twitter_handle_name_services::table, LeftOuter>, Eq<feed_event_wallets::wallet_address, twitter_handle_name_services::wallet_address>>, mint_events::table, LeftOuter>, Eq<feed_events::id, mint_events::feed_event_id>>, offer_events::table, LeftOuter>, Eq<feed_events::id, offer_events::feed_event_id>>, listing_events::table, LeftOuter>, Eq<feed_events::id, listing_events::feed_event_id>>, purchase_events::table, LeftOuter>, Eq<feed_events::id, purchase_events::feed_event_id>>, follow_events::table, LeftOuter>, Eq<feed_events::id, follow_events::feed_event_id>>, Inner>> + QueryFragment<Pg> + QueryId{
    let following_query = graph_connections::table
        .filter(graph_connections::from_account.eq(wallet))
        .filter(graph_connections::disconnected_at.is_null())
        .select(graph_connections::to_account);

    let mut query = feed_event_wallets::table
        .inner_join(feed_events::table)
        .left_join(twitter_handle_name_services::table.on(
            feed_event_wallets::wallet_address.eq(twitter_handle_name_services::wallet_address),
        ))
        .left_join(mint_events::table.on(feed_events::id.eq(mint_events::feed_event_id)))
        .left_join(offer_events::table.on(feed_events::id.eq(offer_events::feed_event_id)))
        .left_join(listing_events::table.on(feed_events::id.eq(listing_events::feed_event_id)))
        .left_join(purchase_events::table.on(feed_events::id.eq(purchase_events::feed_event_id)))
        .left_join(follow_events::table.on(feed_events::id.eq(follow_events::feed_event_id)))
        .filter(feed_event_wallets::wallet_address.eq(any(following_query)))
        .select((
            (feed_events::all_columns),
            (feed_event_wallets::wallet_address),
            (twitter_handle_name_services::twitter_handle.nullable()),
            (mint_events::all_columns.nullable()),
            (offer_events::all_columns.nullable()),
            (listing_events::all_columns.nullable()),
            (purchase_events::all_columns.nullable()),
            (follow_events::all_columns.nullable()),
        ))
        .into_boxed();

    if let Some(event_types) = exclude_types {
        for event_type in event_types {
            query = match event_type {
                EventType::Follow => query.filter(not(follow_events::feed_event_id.is_not_null())),
                EventType::Offer => query.filter(not(offer_events::feed_event_id.is_not_null())),
                EventType::Mint => query.filter(not(mint_events::feed_event_id.is_not_null())),
                EventType::Purchase => {
                    query.filter(not(purchase_events::feed_event_id.is_not_null()))
                },
                EventType::Listing => {
                    query.filter(not(listing_events::feed_event_id.is_not_null()))
                },
            }
        }
    }

    query
        .limit(limit)
        .offset(offset)
        .order(feed_events::created_at.desc())
        .load(conn)
        .context("Failed to load feed events")
}
