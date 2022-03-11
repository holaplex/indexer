//! Query utilities for looking up  metadatas
use diesel::prelude::*;

use crate::{
    db::{
        any,
        models::Nft,
        tables::{
            attributes, listing_receipts, metadata_creators, metadata_jsons, metadatas,
            token_accounts,
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
    /// nft creators
    pub creators: Option<Vec<String>>,
    /// nft attributes
    pub attributes: Option<Vec<AttributeFilter>>,
    /// nft listed with auction house
    pub listed: Option<Vec<String>>,
    /// limit to apply to query
    pub limit: i64,
    /// offset to apply to query
    pub offset: i64,
}

/// Handles queries for NFTs
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn list(
    conn: &Connection,
    ListQueryOptions {
        owners,
        creators,
        attributes,
        listed,
        limit,
        offset,
    }: ListQueryOptions,
) -> Result<Vec<Nft>> {
    let mut query = metadatas::table
        .inner_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .inner_join(
            metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
        )
        .inner_join(
            token_accounts::table.on(metadatas::mint_address.eq(token_accounts::mint_address)),
        )
        .left_outer_join(
            listing_receipts::table.on(metadatas::address.eq(listing_receipts::metadata)),
        )
        .left_outer_join(attributes::table.on(metadatas::address.eq(attributes::metadata_address)))
        .into_boxed();

    if let Some(attributes) = attributes {
        query =
            attributes
                .into_iter()
                .fold(query, |acc, AttributeFilter { trait_type, values }| {
                    acc.filter(
                        attributes::trait_type
                            .eq(trait_type)
                            .and(attributes::value.eq(any(values))),
                    )
                });
    }

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    if let Some(owners) = owners {
        query = query
            .filter(token_accounts::amount.eq(1))
            .filter(token_accounts::owner_address.eq(any(owners)));
    }

    if let Some(listed) = listed {
        query = query
            .filter(listing_receipts::auction_house.eq(any(listed)))
            .filter(listing_receipts::purchase_receipt.is_null())
            .filter(listing_receipts::canceled_at.is_null());
    }

    let rows: Vec<Nft> = query
        .select((
            metadatas::address,
            metadatas::name,
            metadatas::seller_fee_basis_points,
            metadatas::mint_address,
            metadatas::primary_sale_happened,
            metadata_jsons::description,
            metadata_jsons::image,
        ))
        .filter(token_accounts::amount.eq(1))
        .distinct()
        .order_by(metadatas::name.desc())
        .limit(limit)
        .offset(offset)
        .load(conn)
        .context("failed to load nft(s)")?;

    Ok(rows.into_iter().map(Into::into).collect())
}
