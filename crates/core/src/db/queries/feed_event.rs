//! Query utilities for feed events.

use diesel::{prelude::*, sql_types::Timestamp};
use sea_query::{
    Alias, CommonTableExpression, Expr, Iden, Order, PostgresQueryBuilder, Query,
    QueryStatementWriter,
};

use crate::{
    db::{models::CompleteFeedEvent, Connection},
    error::prelude::*,
};

#[derive(Iden)]
enum FeedEvents {
    Table,
    Id,
    CreatedAt,
}

#[derive(Iden)]
enum FeedEventWallets {
    Table,
    FeedEventId,
    WalletAddress,
}

#[derive(Iden)]
enum GraphConnections {
    Table,
    ToAccount,
    FromAccount,
}

#[derive(Iden)]
enum MintEvents {
    Table,
    MetadataAddress,
    FeedEventId,
}

#[derive(Iden)]
enum OfferEvents {
    Table,
    BidReceiptAddress,
    FeedEventId,
    Lifecycle,
}

#[derive(Iden)]
enum ListingEvents {
    Table,
    ListingReceiptAddress,
    FeedEventId,
    Lifecycle,
}

#[derive(Iden)]
enum PurchaseEvents {
    Table,
    PurchaseReceiptAddress,
    FeedEventId,
}

#[derive(Iden)]
enum FollowEvents {
    Table,
    GraphConnectionAddress,
    FeedEventId,
}

#[derive(Iden)]
enum TwitterHandleNameServices {
    Table,
    WalletAddress,
    TwitterHandle,
}

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
#[allow(clippy::too_many_lines)]
pub fn list(
    conn: &Connection,
    limit: u64,
    offset: u64,
    wallet: Option<String>,
    exclude_types: Option<Vec<EventType>>,
) -> Result<Vec<CompleteFeedEvent>> {
    let mut events_query = Query::select()
        .distinct()
        .columns(vec![
            (FeedEvents::Table, FeedEvents::Id),
            (FeedEvents::Table, FeedEvents::CreatedAt),
        ])
        .columns(vec![(
            FeedEventWallets::Table,
            FeedEventWallets::WalletAddress,
        )])
        .column((
            TwitterHandleNameServices::Table,
            TwitterHandleNameServices::TwitterHandle,
        ))
        .column((MintEvents::Table, MintEvents::MetadataAddress))
        .column((
            PurchaseEvents::Table,
            PurchaseEvents::PurchaseReceiptAddress,
        ))
        .column((OfferEvents::Table, OfferEvents::BidReceiptAddress))
        .expr_as(
            Expr::col((OfferEvents::Table, OfferEvents::Lifecycle)),
            Alias::new("offer_lifecycle"),
        )
        .column((ListingEvents::Table, ListingEvents::ListingReceiptAddress))
        .expr_as(
            Expr::col((ListingEvents::Table, ListingEvents::Lifecycle)),
            Alias::new("listing_lifecycle"),
        )
        .column((FollowEvents::Table, FollowEvents::GraphConnectionAddress))
        .from(FeedEvents::Table)
        .inner_join(
            FeedEventWallets::Table,
            Expr::tbl(FeedEventWallets::Table, FeedEventWallets::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .inner_join(
            GraphConnections::Table,
            Expr::tbl(GraphConnections::Table, GraphConnections::ToAccount)
                .equals(FeedEventWallets::Table, FeedEventWallets::WalletAddress),
        )
        .left_join(
            TwitterHandleNameServices::Table,
            Expr::tbl(
                TwitterHandleNameServices::Table,
                TwitterHandleNameServices::WalletAddress,
            )
            .equals(FeedEventWallets::Table, FeedEventWallets::WalletAddress),
        )
        .left_join(
            FollowEvents::Table,
            Expr::tbl(FollowEvents::Table, FollowEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            MintEvents::Table,
            Expr::tbl(MintEvents::Table, MintEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            PurchaseEvents::Table,
            Expr::tbl(PurchaseEvents::Table, PurchaseEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            OfferEvents::Table,
            Expr::tbl(OfferEvents::Table, OfferEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            ListingEvents::Table,
            Expr::tbl(ListingEvents::Table, ListingEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .order_by(FeedEvents::CreatedAt, Order::Desc)
        .clone();

    if let Some(wallet) = wallet {
        events_query.and_where(
            Expr::col((GraphConnections::Table, GraphConnections::FromAccount)).eq(wallet),
        );
    }

    if let Some(event_types) = exclude_types {
        for event_type in event_types {
            match event_type {
                EventType::Follow => events_query.and_where(
                    Expr::col((FollowEvents::Table, FollowEvents::GraphConnectionAddress))
                        .is_null(),
                ),
                EventType::Offer => events_query.and_where(
                    Expr::col((OfferEvents::Table, OfferEvents::BidReceiptAddress)).is_null(),
                ),
                EventType::Mint => events_query.and_where(
                    Expr::col((MintEvents::Table, MintEvents::MetadataAddress)).is_null(),
                ),
                EventType::Purchase => events_query.and_where(
                    Expr::col((
                        PurchaseEvents::Table,
                        PurchaseEvents::PurchaseReceiptAddress,
                    ))
                    .is_null(),
                ),
                EventType::Listing => events_query.and_where(
                    Expr::col((ListingEvents::Table, ListingEvents::ListingReceiptAddress))
                        .is_null(),
                ),
            };
        }
    }

    let events_cte = CommonTableExpression::new()
        .query(events_query)
        .table_name(Alias::new("events"))
        .clone();

    let with_clause = Query::with().cte(events_cte).clone();

    let events_query = Query::select()
        .expr(Expr::asterisk())
        .from(Alias::new("events"))
        .order_by(FeedEvents::CreatedAt, Order::Desc)
        .limit(limit)
        .offset(offset)
        .clone();

    let events_query = events_query
        .with(with_clause)
        .to_string(PostgresQueryBuilder);

    diesel::sql_query(events_query)
        .load(conn)
        .context("Failed to load feed events")
}

/// Return polymorphic list of feed events based on who the wallet is following
///
/// # Errors
/// This function fails if the underlying query fails to execute.
#[allow(clippy::too_many_lines)]
pub fn list_relay(
    conn: &Connection,
    limit: u64,
    is_forward: bool,
    cursor: String,
    wallet: Option<String>,
    include_types: Option<Vec<EventType>>,
) -> Result<Vec<CompleteFeedEvent>> {
    let mut events_query = Query::select()
        .distinct()
        .columns(vec![
            (FeedEvents::Table, FeedEvents::Id),
            (FeedEvents::Table, FeedEvents::CreatedAt),
        ])
        .columns(vec![(
            FeedEventWallets::Table,
            FeedEventWallets::WalletAddress,
        )])
        .column((
            TwitterHandleNameServices::Table,
            TwitterHandleNameServices::TwitterHandle,
        ))
        .column((MintEvents::Table, MintEvents::MetadataAddress))
        .column((
            PurchaseEvents::Table,
            PurchaseEvents::PurchaseReceiptAddress,
        ))
        .column((OfferEvents::Table, OfferEvents::BidReceiptAddress))
        .expr_as(
            Expr::col((OfferEvents::Table, OfferEvents::Lifecycle)),
            Alias::new("offer_lifecycle"),
        )
        .column((ListingEvents::Table, ListingEvents::ListingReceiptAddress))
        .expr_as(
            Expr::col((ListingEvents::Table, ListingEvents::Lifecycle)),
            Alias::new("listing_lifecycle"),
        )
        .column((FollowEvents::Table, FollowEvents::GraphConnectionAddress))
        .from(FeedEvents::Table)
        .inner_join(
            FeedEventWallets::Table,
            Expr::tbl(FeedEventWallets::Table, FeedEventWallets::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .inner_join(
            GraphConnections::Table,
            Expr::tbl(GraphConnections::Table, GraphConnections::ToAccount)
                .equals(FeedEventWallets::Table, FeedEventWallets::WalletAddress),
        )
        .left_join(
            TwitterHandleNameServices::Table,
            Expr::tbl(
                TwitterHandleNameServices::Table,
                TwitterHandleNameServices::WalletAddress,
            )
            .equals(FeedEventWallets::Table, FeedEventWallets::WalletAddress),
        )
        .left_join(
            FollowEvents::Table,
            Expr::tbl(FollowEvents::Table, FollowEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            MintEvents::Table,
            Expr::tbl(MintEvents::Table, MintEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            PurchaseEvents::Table,
            Expr::tbl(PurchaseEvents::Table, PurchaseEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            OfferEvents::Table,
            Expr::tbl(OfferEvents::Table, OfferEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .left_join(
            ListingEvents::Table,
            Expr::tbl(ListingEvents::Table, ListingEvents::FeedEventId)
                .equals(FeedEvents::Table, FeedEvents::Id),
        )
        .order_by(FeedEvents::CreatedAt, Order::Desc)
        .clone();

    if let Some(wallet) = wallet {
        events_query.and_where(
            Expr::col((GraphConnections::Table, GraphConnections::FromAccount)).eq(wallet),
        );
    }

    if is_forward {
        events_query.and_where(Expr::col((FeedEvents::Table, FeedEvents::CreatedAt)).gt(cursor));
    } else {
        events_query.and_where(Expr::col((FeedEvents::Table, FeedEvents::CreatedAt)).lt(cursor));
    }

    events_query.limit(limit);


    if let Some(event_types) = include_types {
        for event_type in event_types {
            match event_type {
                EventType::Follow => events_query.and_where(
                    Expr::col((FollowEvents::Table, FollowEvents::GraphConnectionAddress))
                        .is_not_null(),
                ),
                EventType::Offer => events_query.and_where(
                    Expr::col((OfferEvents::Table, OfferEvents::BidReceiptAddress)).is_not_null(),
                ),
                EventType::Mint => events_query.and_where(
                    Expr::col((MintEvents::Table, MintEvents::MetadataAddress)).is_not_null(),
                ),
                EventType::Purchase => events_query.and_where(
                    Expr::col((
                        PurchaseEvents::Table,
                        PurchaseEvents::PurchaseReceiptAddress,
                    ))
                    .is_not_null(),
                ),
                EventType::Listing => events_query.and_where(
                    Expr::col((ListingEvents::Table, ListingEvents::ListingReceiptAddress))
                        .is_not_null(),
                ),
            };
        }
    }

    events_query.order_by(FeedEvents::CreatedAt, Order::Desc);

    let events_query = events_query
        .to_string(PostgresQueryBuilder);

    diesel::sql_query(events_query)
        .load(conn)
        .context("Failed to load feed events")
}
