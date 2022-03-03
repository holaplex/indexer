//! Query utilities for looking up  metadatas
use diesel::{pg::expression::dsl::any, prelude::*};

use crate::{
    db::{
        models::Nft,
        tables::{attributes, metadata_creators, metadata_jsons, metadatas, token_accounts},
        Connection,
    },
    error::prelude::*,
};
/// Format for incoming filters on attributes
#[derive(Debug)]
pub struct MetadataFilterAttributes {
    /// name of trait
    pub trait_type: String,
    /// array of trait values
    pub values: Vec<String>,
}

/// Handles queries for NFTs
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn load_filtered(
    conn: &Connection,
    owners: Option<Vec<String>>,
    creators: Option<Vec<String>>,
    attributes: Option<Vec<MetadataFilterAttributes>>,
) -> Result<Vec<Nft>> {
    let mut query = metadatas::table
        .left_join(
            metadata_creators::table.on(metadatas::address.eq(metadata_creators::metadata_address)),
        )
        .left_join(
            metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
        )
        .left_join(
            token_accounts::table.on(metadatas::mint_address.eq(token_accounts::mint_address)),
        )
        .into_boxed();

    if let Some(attributes) = attributes {
        query = attributes.into_iter().fold(
            query,
            |acc, MetadataFilterAttributes { trait_type, values }| {
                let sub = attributes::table
                    .select(attributes::metadata_address)
                    .filter(
                        attributes::trait_type
                            .eq(trait_type)
                            .and(attributes::value.eq(any(values))),
                    );

                acc.filter(metadatas::address.eq(any(sub)))
            },
        );
    }

    if let Some(creators) = creators {
        query = query.filter(metadata_creators::creator_address.eq(any(creators)));
    }

    if let Some(owners) = owners {
        query = query
            .filter(token_accounts::amount.eq(1))
            .filter(token_accounts::owner_address.eq(any(owners)));
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
        .load(conn)
        .context("failed to load nft(s)")?;

    Ok(rows.into_iter().map(Into::into).collect())
}
