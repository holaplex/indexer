//! Query utilities for feed events.

use std::str::FromStr;

use diesel::{
    expression::{nullable::Nullable, operators::Eq, AsExpression, NonAggregate},
    prelude::*,
    query_builder::{QueryFragment, QueryId},
    query_source::joins::{Inner, Join, JoinOn, LeftOuter},
    sql_types::Text,
    AppearsOnTable,
};
use sea_query::{Iden, Query, Expr};

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

#[derive(Iden)]
enum FeedEvents {
    Table,
    Id,
    CreatedAt
}

#[derive(Iden)]
enum FeedEventWallets {
    Table,
    WalletAddress,
    FeedEventId
}

#[derive(Iden)]
enum TwitterHandleNameServices {
    Table,
    Address,
    WalletAddress,
    TwitterHandle,
    Slot,
    FromBonfida,
    FromCardinal,
    WriteVersion
}


#[derive(Iden)]
enum MintEvents {
    Table,
    MetadataAddress,
    FeedEventId
}

#[derive(Iden)]
enum OfferEvents {
    Table,
    BidReceiptAddress,
    FeedEventId
}


#[derive(Iden)]
enum ListingEvents {
    Table,
    ListingReceiptAddress,
    FeedEventId,
    Lifecycle
}

#[derive(Iden)]
enum PurchaseEvents {
    Table,
    PurchseReceiptAddress,
    FeedEventId
}


#[derive(Iden)]
enum FollowEvents {
    Table,
    GraphConnectionAddress,
    FeedEventId
}

#[derive(Iden)]
enum GraphConnections {
    Table,
    Address,
    FromAccount,
    ToAccount,
    ConnectedAt,
    DisconnectedAt
}

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

// When you add a new type, be sure to add it to the iterator implementation below
#[derive(Debug, Clone, Copy)]
pub enum FeedEventType {
    /// Mint Events
    Mint,

    /// Offer Events
    Offer,

    /// Listing Events
    Listing,

    /// Purchase Events
    Purchase,

    /// Follow Events
    Follow
}


impl FeedEventType {
    pub fn iterator() -> impl Iterator<Item = FeedEventType> {
        [FeedEventType::Mint, FeedEventType::Offer, FeedEventType::Listing, FeedEventType::Purchase, FeedEventType::Follow].iter().copied()
    }
}


impl FromStr for FeedEventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "mint" => Ok(FeedEventType::Mint),
            "offer" => Ok(FeedEventType::Offer),
            "listing" => Ok(FeedEventType::Listing),
            "purchase" => Ok(FeedEventType::Purchase),
            "follow" => Ok(FeedEventType::Follow),
            _ => Err(format!("Unknown event type {}", &s))
        }
    }
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
    types: Option<Vec<FeedEventType>>
) -> Result<
    Vec<Columns>,
> where <W as AsExpression<Text>>::Expression: NonAggregate + AppearsOnTable<Join<graph_connections::table, JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<feed_event_wallets::table, feed_events::table, Inner>, Eq<Nullable<feed_event_wallets::feed_event_id>, Nullable<feed_events::id>>>, twitter_handle_name_services::table, LeftOuter>, Eq<feed_event_wallets::wallet_address, twitter_handle_name_services::wallet_address>>, mint_events::table, LeftOuter>, Eq<feed_events::id, mint_events::feed_event_id>>, offer_events::table, LeftOuter>, Eq<feed_events::id, offer_events::feed_event_id>>, listing_events::table, LeftOuter>, Eq<feed_events::id, listing_events::feed_event_id>>, purchase_events::table, LeftOuter>, Eq<feed_events::id, purchase_events::feed_event_id>>, follow_events::table, LeftOuter>, Eq<feed_events::id, follow_events::feed_event_id>>, Inner>> + QueryFragment<Pg> + QueryId{
    let types = types.unwrap_or(FeedEventType::iterator().collect());
    
    let following_query = Query::select()
        .column((GraphConnections::Table, GraphConnections::ToAccount))
        .and_where(Expr::tbl(GraphConnections::Table, GraphConnections::FromAccount).equals(wallet));

    // TODO
    feed_event_wallets::table
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
        .limit(limit)
        .offset(offset)
        .order(feed_events::created_at.desc())
        .load(conn)
        .context("Failed to load feed events")
}
