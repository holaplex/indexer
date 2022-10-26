//! Query utilities for looking up  metadatas
use diesel::prelude::*;
use sea_query::{Condition, Expr, Iden, Order, PostgresQueryBuilder, Query, Value};

use crate::{
    db::{models::Nft, Connection},
    error::prelude::*,
    prelude::Utc,
};

/// Format for incoming filters on attributes
#[derive(Debug)]
pub struct AttributeFilter {
    /// name of trait
    pub trait_type: String,
    /// array of trait values
    pub values: Vec<String>,
}

#[derive(Iden)]
enum Metadatas {
    Table,
    Address,
    Name,
    MintAddress,
    PrimarySaleHappened,
    SellerFeeBasisPoints,
    UpdateAuthorityAddress,
    Uri,
    Slot,
    BurnedAt,
}

#[derive(Iden)]
enum CollectionMints {
    Table,
    CollectionId,
    Mint,
}

#[derive(Iden)]
enum MetadataJsons {
    Table,
    MetadataAddress,
    Description,
    Image,
    AnimationUrl,
    ExternalUrl,
    Category,
    Model,
}

#[derive(Iden)]
enum CurrentMetadataOwners {
    Table,
    OwnerAddress,
    MintAddress,
    TokenAccountAddress,
}

#[derive(Iden)]
enum Listings {
    Table,
    Price,
    Metadata,
    AuctionHouse,
    Seller,
    PurchaseId,
    CanceledAt,
    Expiry,
}

/// Input parameters for the [`collection_nfts`] query.
#[derive(Debug)]
pub struct CollectionListedNftOptions {
    /// Collection address
    pub collection: String,
    /// Auction house of the collection
    pub auction_house: Option<String>,
    /// Limit the number of returned rows
    pub limit: u64,
    /// Skip the first `n` resulting rows
    pub offset: u64,
}

/// Handles queries for a Collection Listed Nfts
///
/// # Errors
/// returns an error when the underlying queries throw an error
#[allow(clippy::too_many_lines)]
pub fn list<O: Into<Value>>(
    conn: &Connection,
    options: CollectionListedNftOptions,
    opensea_auction_house: O,
) -> Result<Vec<Nft>> {
    let CollectionListedNftOptions {
        collection,
        auction_house,
        limit,
        offset,
    } = options;

    let current_time = Utc::now().naive_utc();

    let query = Query::select()
        .columns(vec![
            (Metadatas::Table, Metadatas::Address),
            (Metadatas::Table, Metadatas::Name),
            (Metadatas::Table, Metadatas::SellerFeeBasisPoints),
            (Metadatas::Table, Metadatas::UpdateAuthorityAddress),
            (Metadatas::Table, Metadatas::MintAddress),
            (Metadatas::Table, Metadatas::PrimarySaleHappened),
            (Metadatas::Table, Metadatas::Uri),
            (Metadatas::Table, Metadatas::Slot),
        ])
        .columns(vec![
            (MetadataJsons::Table, MetadataJsons::Description),
            (MetadataJsons::Table, MetadataJsons::Image),
            (MetadataJsons::Table, MetadataJsons::AnimationUrl),
            (MetadataJsons::Table, MetadataJsons::ExternalUrl),
            (MetadataJsons::Table, MetadataJsons::Category),
            (MetadataJsons::Table, MetadataJsons::Model),
        ])
        .columns(vec![(
            CurrentMetadataOwners::Table,
            CurrentMetadataOwners::TokenAccountAddress,
        )])
        .from(MetadataJsons::Table)
        .inner_join(
            Metadatas::Table,
            Expr::tbl(MetadataJsons::Table, MetadataJsons::MetadataAddress)
                .equals(Metadatas::Table, Metadatas::Address),
        )
        .inner_join(
            CurrentMetadataOwners::Table,
            Expr::tbl(Metadatas::Table, Metadatas::MintAddress).equals(
                CurrentMetadataOwners::Table,
                CurrentMetadataOwners::MintAddress,
            ),
        )
        .inner_join(
            CollectionMints::Table,
            Expr::tbl(CollectionMints::Table, CollectionMints::Mint)
                .equals(Metadatas::Table, Metadatas::MintAddress),
        )
        .left_join(
            Listings::Table,
            Condition::all()
                .add(
                    Expr::tbl(Listings::Table, Listings::Metadata)
                        .equals(Metadatas::Table, Metadatas::Address),
                )
                .add(Expr::tbl(Listings::Table, Listings::Seller).equals(
                    CurrentMetadataOwners::Table,
                    CurrentMetadataOwners::OwnerAddress,
                ))
                .add(Expr::tbl(Listings::Table, Listings::PurchaseId).is_null())
                .add(Expr::tbl(Listings::Table, Listings::CanceledAt).is_null())
                .add(Expr::tbl(Listings::Table, Listings::AuctionHouse).ne(opensea_auction_house))
                .add(
                    Expr::tbl(Listings::Table, Listings::Expiry)
                        .is_null()
                        .or(Expr::tbl(Listings::Table, Listings::Expiry).gt(current_time)),
                )
                .add_option(auction_house.map(|auction_house| {
                    Expr::col((Listings::Table, Listings::AuctionHouse)).eq(auction_house)
                })),
        )
        .and_where(Expr::col(Metadatas::BurnedAt).is_null())
        .and_where(
            Expr::col((CollectionMints::Table, CollectionMints::CollectionId)).eq(collection),
        )
        .limit(limit)
        .offset(offset)
        .order_by((Listings::Table, Listings::Price), Order::Asc)
        .take();

    let query = query.to_string(PostgresQueryBuilder);

    diesel::sql_query(query)
        .load(conn)
        .context("Failed to load collection listed nft(s)")
}
