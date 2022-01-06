//! Query utilities for looking up the edition associated with a metadata
//! address.

use std::borrow::Cow;

use anyhow::Context;
use diesel::prelude::*;

use crate::{
    db::{
        models::{Edition, MasterEdition},
        tables::{editions, master_editions, metadatas},
        Connection,
    },
    error::prelude::*,
};

/// Edition information for a metadata address
#[derive(Debug, Clone)]
pub enum MetadataEdition<'a> {
    /// A non-master edition
    Edition(Edition<'a>),
    /// A master edition
    MasterEdition(MasterEdition<'a>),
}

/// Load the edition for a metadata, whether it be from the `editions` or
/// `master_editions` tables
///
/// # Errors
/// This function returns an error if either of the two underlying database
/// queries fail.
pub fn load<'a>(
    metadata_address: &'a str,
    conn: &Connection,
) -> Result<Option<MetadataEdition<'a>>> {
    type Cols = (
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
        Option<i64>,
        Option<i64>,
    );

    let metas = metadatas::table
        .filter(metadatas::address.eq(metadata_address))
        .left_join(editions::table)
        .left_join(master_editions::table)
        .limit(1)
        .select((
            editions::address.nullable(),
            editions::parent_address.nullable(),
            editions::edition.nullable(),
            master_editions::address.nullable(),
            master_editions::supply.nullable(),
            master_editions::max_supply.nullable(),
        ))
        .load::<Cols>(conn)
        .context("Failed to load editions")?;

    let (edition_addr, edition_parent, edition_ord, master_addr, master_supply, master_max) =
        if metas.len() == 1 {
            metas.into_iter().next().unwrap_or_else(|| unreachable!())
        } else {
            bail!("Invalid metadata address");
        };

    Ok(edition_addr
        .map(|address| {
            MetadataEdition::Edition(Edition {
                address: Cow::Owned(address),
                parent_address: edition_parent.map_or_else(|| unreachable!(), Cow::Owned),
                edition: edition_ord.unwrap_or_else(|| unreachable!()),
                metadata_address: Cow::Borrowed(metadata_address),
            })
        })
        .or_else(|| {
            master_addr.map(|address| {
                MetadataEdition::MasterEdition(MasterEdition {
                    address: Cow::Owned(address),
                    supply: master_supply.unwrap_or_else(|| unreachable!()),
                    max_supply: master_max,
                    metadata_address: Cow::Borrowed(metadata_address),
                })
            })
        }))
}
