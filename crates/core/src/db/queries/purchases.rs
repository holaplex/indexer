//! Query utilities for purchases.

use diesel::prelude::*;
use sea_query::{Expr, Iden, Order, PostgresQueryBuilder, Query};

use crate::{
    db::{models::Purchase, Connection},
    error::prelude::*,
};

#[derive(Iden)]
enum Purchases {
    Table,
    Id,
    Buyer,
    Seller,
    AuctionHouse,
    Metadata,
    TokenSize,
    Price,
    CreatedAt,
    Slot,
    WriteVersion,
    MarketplaceProgram,
}

/// Return list of recent purchases
///
/// # Errors
/// This function fails if the underlying query fails to execute.
#[allow(clippy::too_many_lines)]
pub fn list(
    conn: &Connection,
    limit: u64,
    offset: u64,
    auction_houses: Option<Vec<String>>,
    marketplace_programs: Option<Vec<String>>,
) -> Result<Vec<Purchase>> {
    let mut query = Query::select()
        .distinct()
        .columns(vec![
            (Purchases::Table, Purchases::Id),
            (Purchases::Table, Purchases::Buyer),
            (Purchases::Table, Purchases::Seller),
            (Purchases::Table, Purchases::AuctionHouse),
            (Purchases::Table, Purchases::Metadata),
            (Purchases::Table, Purchases::TokenSize),
            (Purchases::Table, Purchases::Price),
            (Purchases::Table, Purchases::CreatedAt),
            (Purchases::Table, Purchases::Slot),
            (Purchases::Table, Purchases::WriteVersion),
            (Purchases::Table, Purchases::MarketplaceProgram),
        ])
        .from(Purchases::Table)
        .clone();

    if let Some(auction_houses) = auction_houses {
        query.and_where(
            Expr::col((Purchases::Table, Purchases::AuctionHouse)).is_in(auction_houses),
        );
    }

    if let Some(marketplace_programs) = marketplace_programs {
        query.and_where(
            Expr::col((Purchases::Table, Purchases::MarketplaceProgram))
                .is_in(marketplace_programs),
        );
    }

    query
        .order_by(Purchases::CreatedAt, Order::Desc)
        .limit(limit)
        .offset(offset);

    let query = query.to_string(PostgresQueryBuilder);

    diesel::sql_query(query)
        .load(conn)
        .context("Failed to load purchases")
}
