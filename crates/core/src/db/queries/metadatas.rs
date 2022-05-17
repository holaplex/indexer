//! Query utilities for looking up  metadatas

use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text},
};

use crate::{
    db::{
        any,
        models::{Nft, NftActivity},
        not,
        tables::{
            attributes, bid_receipts, current_metadata_owners, listing_receipts,
            metadata_collection_keys, metadata_creators, metadata_jsons, metadatas,
        },
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

/// List query options
#[derive(Debug)]
pub struct ListQueryOptions {
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
    pub limit: i64,
    /// offset to apply to query
    pub offset: i64,
}

/// The column set for an NFT
pub type NftColumns = (
    metadatas::address,
    metadatas::name,
    metadatas::seller_fee_basis_points,
    metadatas::mint_address,
    metadatas::primary_sale_happened,
    metadatas::uri,
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
    let listed = listed.unwrap_or(false);

    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(
            metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
        )
        .inner_join(
            current_metadata_owners::table
                .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
        )
        .left_outer_join(
            listing_receipts::table.on(metadatas::address.eq(listing_receipts::metadata)),
        )
        .left_outer_join(bid_receipts::table.on(metadatas::address.eq(bid_receipts::metadata)))
        .left_outer_join(
            metadata_collection_keys::table
                .on(metadatas::address.eq(metadata_collection_keys::metadata_address)),
        )
        .distinct()
        .select((NftColumns::default(), listing_receipts::price.nullable()))
        .order_by((listing_receipts::price.asc(), metadatas::name.asc()))
        .into_boxed();

    if let Some(auction_houses) = auction_houses {
        query = query.filter(listing_receipts::auction_house.eq(any(auction_houses)));
        query = query.or_filter(listing_receipts::auction_house.is_null());
    }

    if let Some(attributes) = attributes {
        query =
            attributes
                .into_iter()
                .fold(query, |acc, AttributeFilter { trait_type, values }| {
                    let sub = attributes::table
                        .select(attributes::metadata_address)
                        .filter(
                            attributes::trait_type
                                .eq(trait_type)
                                .and(attributes::value.eq(any(values))),
                        );

                    acc.filter(metadatas::address.eq(any(sub)))
                });
    }

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
        query = query.filter(metadata_creators::verified.eq(true));
    }

    if let Some(collection) = collection {
        query = query.filter(metadata_collection_keys::collection_address.eq(collection));
    }

    if let Some(owners) = owners {
        query = query.filter(current_metadata_owners::owner_address.eq(any(owners)));
    }

    if let Some(offerers) = offerers {
        query = query
            .filter(bid_receipts::buyer.eq(any(offerers)))
            .filter(bid_receipts::purchase_receipt.is_null())
            .filter(bid_receipts::canceled_at.is_null());
    }

    if listed {
        query = query.filter(not(listing_receipts::price.is_null()));
    }

    let rows: Vec<(Nft, Option<i64>)> = query
        .filter(listing_receipts::purchase_receipt.is_null())
        .filter(listing_receipts::canceled_at.is_null())
        .limit(limit)
        .offset(offset)
        .load(conn)
        .context("failed to load nft(s)")?;

    Ok(rows.into_iter().map(|(nft, _)| nft).collect())
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
