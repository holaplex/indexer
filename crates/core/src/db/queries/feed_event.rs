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
        tables::{feed_event_wallets, feed_events, graph_connections, mint_events, offer_events},
        Connection,
    },
    error::prelude::*,
};

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
    Vec<(
        models::FeedEvent,
        Option<models::MintEvent>,
        Option<models::OfferEvent>,
    )>,
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
        >,
{
    let following_query = graph_connections::table
        .filter(graph_connections::from_account.eq(wallet.clone()))
        .select(graph_connections::to_account);

    let rows: Vec<(
        models::FeedEvent,
        Option<models::MintEvent>,
        Option<models::OfferEvent>,
    )> = feed_event_wallets::table
        .inner_join(feed_events::table)
        .left_join(mint_events::table.on(feed_events::id.eq(mint_events::feed_event_id)))
        .left_join(offer_events::table.on(feed_events::id.eq(offer_events::feed_event_id)))
        .filter(feed_event_wallets::wallet_address.eq(wallet))
        .or_filter(feed_event_wallets::wallet_address.eq(any(following_query)))
        .select((
            (feed_events::all_columns),
            (mint_events::all_columns.nullable()),
            (offer_events::all_columns.nullable()),
        ))
        .limit(limit)
        .offset(offset)
        .order(feed_events::created_at.desc())
        .load(conn)
        .context("Failed to load feed events")?;

    Ok(rows)
}
