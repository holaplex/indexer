//! Query utilities for feed events.

use diesel::{
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
            mint_events, offer_events, purchase_events,
        },
        Connection,
    },
    error::prelude::*,
};

/// join of event tables into a single event type
pub type Columns<'a> = (
    models::FeedEvent<'a>,
    Option<models::MintEvent<'a>>,
    Option<models::OfferEvent<'a>>,
    Option<models::ListingEvent<'a>>,
    Option<models::PurchaseEvent<'a>>,
    Option<models::FollowEvent<'a>>,
);

/// Return polymorphic list of feed events based on the wallet
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn list<W: Clone + AsExpression<Text>>(
    conn: &Connection,
    wallet: W,
    limit: i64,
    offset: i64,
) -> Result<
    Vec<Columns>,
>
where
    W::Expression: NonAggregate
        + QueryId
        + QueryFragment<Pg>
        + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<feed_event_wallets::table, feed_events::table, Inner>,
                        Eq<Nullable<feed_event_wallets::feed_event_id>, Nullable<feed_events::id>>,
                    >,
                    mint_events::table,
                    LeftOuter,
                >,
                Eq<feed_events::id, mint_events::feed_event_id>,
            >,
        > + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<
                            JoinOn<
                                Join<feed_event_wallets::table, feed_events::table, Inner>,
                                Eq<
                                    Nullable<feed_event_wallets::feed_event_id>,
                                    Nullable<feed_events::id>,
                                >,
                            >,
                            mint_events::table,
                            LeftOuter,
                        >,
                        Eq<feed_events::id, mint_events::feed_event_id>,
                    >,
                    offer_events::table,
                    LeftOuter,
                >,
                Eq<feed_events::id, offer_events::feed_event_id>,
            >,
        > + AppearsOnTable<
            Join<
                graph_connections::table,
                JoinOn<
                    Join<
                        JoinOn<
                            Join<
                                JoinOn<
                                    Join<feed_event_wallets::table, feed_events::table, Inner>,
                                    Eq<
                                        Nullable<feed_event_wallets::feed_event_id>,
                                        Nullable<feed_events::id>,
                                    >,
                                >,
                                mint_events::table,
                                LeftOuter,
                            >,
                            Eq<feed_events::id, mint_events::feed_event_id>,
                        >,
                        offer_events::table,
                        LeftOuter,
                    >,
                    Eq<feed_events::id, offer_events::feed_event_id>,
                >,
                Inner,
            >,
        > + AppearsOnTable<
            Join<
                graph_connections::table,
                JoinOn<
                    Join<
                        JoinOn<
                            Join<
                                JoinOn<
                                    Join<
                                        JoinOn<
                                            Join<
                                                JoinOn<
                                                    Join<
                                                        feed_event_wallets::table,
                                                        feed_events::table,
                                                        Inner,
                                                    >,
                                                    Eq<
                                                        Nullable<feed_event_wallets::feed_event_id>,
                                                        Nullable<feed_events::id>,
                                                    >,
                                                >,
                                                mint_events::table,
                                                LeftOuter,
                                            >,
                                            Eq<feed_events::id, mint_events::feed_event_id>,
                                        >,
                                        offer_events::table,
                                        LeftOuter,
                                    >,
                                    Eq<feed_events::id, offer_events::feed_event_id>,
                                >,
                                listing_events::table,
                                LeftOuter,
                            >,
                            Eq<feed_events::id, listing_events::feed_event_id>,
                        >,
                        purchase_events::table,
                        LeftOuter,
                    >,
                    Eq<feed_events::id, purchase_events::feed_event_id>,
                >,
                Inner,
            >,
        > + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<
                            JoinOn<
                                Join<
                                    JoinOn<
                                        Join<
                                            JoinOn<
                                                Join<
                                                    feed_event_wallets::table,
                                                    feed_events::table,
                                                    Inner,
                                                >,
                                                Eq<
                                                    Nullable<
                                                        feed_event_wallets::feed_event_id,
                                                    >,
                                                    Nullable<
                                                        feed_events::id,
                                                    >,
                                                >,
                                            >,
                                            mint_events::table,
                                            LeftOuter,
                                        >,
                                        Eq<feed_events::id, mint_events::feed_event_id>,
                                    >,
                                    offer_events::table,
                                    LeftOuter,
                                >,
                                Eq<feed_events::id, offer_events::feed_event_id>,
                            >,
                            listing_events::table,
                            LeftOuter,
                        >,
                        Eq<feed_events::id, listing_events::feed_event_id>,
                    >,
                    purchase_events::table,
                    LeftOuter,
                >,
                Eq<feed_events::id, purchase_events::feed_event_id>,
            >,
        > + AppearsOnTable<
            JoinOn<
                Join<
                    JoinOn<
                        Join<
                            JoinOn<
                                Join<
                                    JoinOn<
                                        Join<
                                            JoinOn<
                                                Join<
                                                    JoinOn<
                                                        Join<
                                                            feed_event_wallets::table,
                                                            feed_events::table,
                                                            Inner,
                                                        >,
                                                        Eq<
                                                            Nullable<
                                                                feed_event_wallets::feed_event_id,
                                                            >,
                                                            Nullable<
                                                                feed_events::id,
                                                            >,
                                                        >,
                                                    >,
                                                    mint_events::table,
                                                    LeftOuter,
                                                >,
                                                Eq<
                                                    feed_events::id,
                                                    mint_events::feed_event_id,
                                                >,
                                            >,
                                            offer_events::table,
                                            LeftOuter,
                                        >,
                                        Eq<
                                            feed_events::id,
                                            offer_events::feed_event_id,
                                        >,
                                    >,
                                    listing_events::table,
                                    LeftOuter,
                                >,
                                Eq<
                                    feed_events::id,
                                    listing_events::feed_event_id,
                                >,
                            >,
                            purchase_events::table,
                            LeftOuter,
                        >,
                        Eq<
                            feed_events::id,
                            purchase_events::feed_event_id,
                        >,
                    >,
                    follow_events::table,
                    LeftOuter,
                >,
                Eq<feed_events::id, follow_events::feed_event_id>,
            >,
        > + AppearsOnTable<Join<graph_connections::table, JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<JoinOn<Join<feed_event_wallets::table, feed_events::table, Inner>, Eq<Nullable<feed_event_wallets::feed_event_id>, Nullable<feed_events::id>>>, mint_events::table, LeftOuter>, Eq<feed_events::id, mint_events::feed_event_id>>, offer_events::table, LeftOuter>, Eq<feed_events::id, offer_events::feed_event_id>>, listing_events::table, LeftOuter>, Eq<feed_events::id, listing_events::feed_event_id>>, purchase_events::table, LeftOuter>, Eq<feed_events::id, purchase_events::feed_event_id>>, follow_events::table, LeftOuter>, Eq<feed_events::id, follow_events::feed_event_id>>, Inner>>
{
    let following_query = graph_connections::table
        .filter(graph_connections::from_account.eq(wallet.clone()))
        .select(graph_connections::to_account);

    feed_event_wallets::table
        .inner_join(feed_events::table)
        .left_join(mint_events::table.on(feed_events::id.eq(mint_events::feed_event_id)))
        .left_join(offer_events::table.on(feed_events::id.eq(offer_events::feed_event_id)))
        .left_join(listing_events::table.on(feed_events::id.eq(listing_events::feed_event_id)))
        .left_join(purchase_events::table.on(feed_events::id.eq(purchase_events::feed_event_id)))
        .left_join(follow_events::table.on(feed_events::id.eq(follow_events::feed_event_id)))
        .filter(feed_event_wallets::wallet_address.eq(wallet))
        .or_filter(feed_event_wallets::wallet_address.eq(any(following_query)))
        .select((
            (feed_events::all_columns),
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
