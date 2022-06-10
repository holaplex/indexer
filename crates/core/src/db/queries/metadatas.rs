//! Query utilities for looking up  metadatas
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text},
};
use sea_query::{
    Alias, Condition, DynIden, Expr, Iden, JoinType, Order, PostgresQueryBuilder, Query, SeaRc,
};

use crate::{
    db::{
        models::{Nft, NftActivity},
        tables::{metadata_jsons, metadatas},
        Connection,
    },
    error::prelude::*,
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
}

#[derive(Iden)]
enum MetadataJsons {
    Table,
    MetadataAddress,
    Description,
    Image,
    Category,
    Model,
}

#[derive(Iden)]
enum CurrentMetadataOwners {
    Table,
    OwnerAddress,
    MintAddress,
}

#[derive(Iden)]
enum ListingReceipts {
    Table,
    Price,
    Metadata,
    AuctionHouse,
    Seller,
    PurchaseReceipt,
    CanceledAt,
}

#[derive(Iden)]
enum MetadataCreators {
    Table,
    CreatorAddress,
    MetadataAddress,
    Verified,
}

#[derive(Iden)]

enum BidReceipts {
    Table,
    Buyer,
    Price,
    Metadata,
    CanceledAt,
    PurchaseReceipt,
    AuctionHouse,
}

#[derive(Iden)]
enum Attributes {
    Table,
    MetadataAddress,
    TraitType,
    Value,
}

#[derive(Iden)]
enum MetadataCollectionKeys {
    Table,
    MetadataAddress,
    CollectionAddress,
}

/// List query options
#[derive(Debug)]
pub struct ListQueryOptions {
    /// NFT metadata addresses
    pub addresses: Option<Vec<String>>,
    /// nft owners
    pub owners: Option<Vec<String>>,
    /// auction houses
    pub auction_houses: Option<Vec<String>>,
    /// nft creators
    pub creators: Option<Vec<String>>,
    /// offerers who provided offers on nft
    pub offerers: Option<Vec<String>>,
    /// nft attributes
    pub attributes: Option<Vec<AttributeFilter>>,
    /// nfts listed for sale
    pub listed: Option<bool>,
    /// nft in a specific colleciton
    pub collection: Option<String>,
    /// limit to apply to query
    pub limit: u64,
    /// offset to apply to query
    pub offset: u64,
}

/// The column set for an NFT
pub type NftColumns = (
    metadatas::address,
    metadatas::name,
    metadatas::seller_fee_basis_points,
    metadatas::mint_address,
    metadatas::primary_sale_happened,
    metadatas::update_authority_address,
    metadatas::uri,
    metadatas::slot,
    metadata_jsons::description,
    metadata_jsons::image,
    metadata_jsons::category,
    metadata_jsons::model,
);

/// Handles queries for NFTs
///
/// # Errors
/// returns an error when the underlying queries throw an error
#[allow(clippy::too_many_lines)]
pub fn list(
    conn: &Connection,
    ListQueryOptions {
        addresses,
        owners,
        creators,
        auction_houses,
        offerers,
        attributes,
        listed,
        collection,
        limit,
        offset,
    }: ListQueryOptions,
) -> Result<Vec<Nft>> {
    let mut listing_receipts_query = Query::select()
        .columns(vec![
            (ListingReceipts::Table, ListingReceipts::Metadata),
            (ListingReceipts::Table, ListingReceipts::Price),
            (ListingReceipts::Table, ListingReceipts::Seller),
        ])
        .from(ListingReceipts::Table)
        .order_by(
            (ListingReceipts::Table, ListingReceipts::Price),
            Order::Desc,
        )
        .cond_where(
            Condition::all()
                .add(Expr::tbl(ListingReceipts::Table, ListingReceipts::PurchaseReceipt).is_null())
                .add(Expr::tbl(ListingReceipts::Table, ListingReceipts::CanceledAt).is_null()),
        )
        .take();

    if let Some(auction_houses) = auction_houses.clone() {
        listing_receipts_query.and_where(
            Expr::col((ListingReceipts::Table, ListingReceipts::AuctionHouse))
                .is_in(auction_houses),
        );
    }

    let mut query = Query::select()
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
            (MetadataJsons::Table, MetadataJsons::Category),
            (MetadataJsons::Table, MetadataJsons::Model),
        ])
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
        .join_lateral(
            JoinType::LeftJoin,
            listing_receipts_query.take(),
            ListingReceipts::Table,
            Condition::all()
                .add(
                    Expr::tbl(ListingReceipts::Table, ListingReceipts::Metadata)
                        .equals(Metadatas::Table, Metadatas::Address),
                )
                .add(
                    Expr::tbl(ListingReceipts::Table, ListingReceipts::Seller).equals(
                        CurrentMetadataOwners::Table,
                        CurrentMetadataOwners::OwnerAddress,
                    ),
                ),
        )
        .limit(limit)
        .offset(offset)
        .order_by((ListingReceipts::Table, ListingReceipts::Price), Order::Asc)
        .take();

    if let Some(addresses) = addresses {
        query.and_where(Expr::col(Metadatas::Address).is_in(addresses));
    }

    if let Some(owners) = owners {
        query.and_where(Expr::col(CurrentMetadataOwners::OwnerAddress).is_in(owners));
    }

    if let Some(creators) = creators {
        query
            .inner_join(
                MetadataCreators::Table,
                Expr::tbl(Metadatas::Table, Metadatas::Address)
                    .equals(MetadataCreators::Table, MetadataCreators::MetadataAddress),
            )
            .and_where(Expr::col(MetadataCreators::CreatorAddress).is_in(creators))
            .and_where(Expr::col(MetadataCreators::Verified).eq(true));
    }

    if let Some(listed) = listed {
        query.conditions(
            listed,
            |q| {
                q.and_where(
                    Expr::col((ListingReceipts::Table, ListingReceipts::Price)).is_not_null(),
                );
            },
            |q| {
                q.and_where(Expr::col((ListingReceipts::Table, ListingReceipts::Price)).is_null());
            },
        );
    }

    if let Some(offerers) = offerers {
        let mut bid_receipts_query = Query::select()
            .columns(vec![
                (BidReceipts::Table, BidReceipts::Metadata),
                (BidReceipts::Table, BidReceipts::Price),
            ])
            .from(BidReceipts::Table)
            .cond_where(
                Condition::all()
                    .add(Expr::col((BidReceipts::Table, BidReceipts::Buyer)).is_in(offerers))
                    .add(Expr::tbl(BidReceipts::Table, BidReceipts::PurchaseReceipt).is_null())
                    .add(Expr::tbl(BidReceipts::Table, BidReceipts::CanceledAt).is_null())
                    .add(
                        Expr::tbl(BidReceipts::Table, BidReceipts::Metadata)
                            .equals(Metadatas::Table, Metadatas::Address),
                    ),
            )
            .take();

        if let Some(auction_houses) = auction_houses {
            bid_receipts_query.and_where(
                Expr::col((BidReceipts::Table, BidReceipts::AuctionHouse)).is_in(auction_houses),
            );
        }

        query.join_lateral(
            JoinType::InnerJoin,
            bid_receipts_query.take(),
            BidReceipts::Table,
            Expr::tbl(BidReceipts::Table, BidReceipts::Metadata)
                .equals(Metadatas::Table, Metadatas::Address),
        );
    }

    if let Some(attributes) = attributes {
        for AttributeFilter { trait_type, values } in attributes {
            let alias = format!("attributes_{}", trait_type);
            let alias: DynIden = SeaRc::new(Alias::new(&alias));

            query.join_lateral(
                JoinType::InnerJoin,
                Query::select()
                    .from(Attributes::Table)
                    .column((Attributes::Table, Attributes::MetadataAddress))
                    .cond_where(
                        Condition::all()
                            .add(Expr::col(Attributes::TraitType).eq(trait_type))
                            .add(Expr::col(Attributes::Value).is_in(values)),
                    )
                    .take(),
                alias.clone(),
                Expr::tbl(alias, Attributes::MetadataAddress)
                    .equals(Metadatas::Table, Metadatas::Address),
            );
        }
    }

    if let Some(collection) = collection {
        query.inner_join(
            MetadataCollectionKeys::Table,
            Expr::tbl(
                MetadataCollectionKeys::Table,
                MetadataCollectionKeys::MetadataAddress,
            )
            .equals(Metadatas::Table, Metadatas::Address),
        );

        query.and_where(
            Expr::col((
                MetadataCollectionKeys::Table,
                MetadataCollectionKeys::CollectionAddress,
            ))
            .eq(collection),
        );
    }

    let query = query.to_string(PostgresQueryBuilder);

    diesel::sql_query(query)
        .load(conn)
        .context("Failed to load nft(s)")
}

const ACTIVITES_QUERY: &str = r"
    SELECT listing_receipts.address as address, metadata, auction_house, price, auction_house, created_at,
    array[seller] as wallets,
    array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
    'listing' as activity_type
        FROM listing_receipts
        LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listing_receipts.seller)
        WHERE metadata = ANY($1)
    UNION
    SELECT purchase_receipts.address as address, metadata, auction_house, price, auction_house, created_at,
    array[seller, buyer] as wallets,
    array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
    'purchase' as activity_type
        FROM purchase_receipts
        LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchase_receipts.seller)
        LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchase_receipts.buyer)
        WHERE metadata = ANY($1)
    ORDER BY created_at DESC;
 -- $1: addresses::text[]";

/// Load listing and sales activity for nfts
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn activities(
    conn: &Connection,
    addresses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<NftActivity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(addresses)
        .load(conn)
        .context("Failed to load nft(s) activities")
}
