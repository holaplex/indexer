//! Query utilities for looking up the edition associated with a metadata
//! address.

use anyhow::Context;
use diesel::prelude::*;
use solana_sdk::pubkey::Pubkey;

use crate::{
    db::{
        models::{Edition, MasterEdition},
        tables::{editions, master_editions},
        Connection,
    },
    error::Result,
    pubkeys::find_edition,
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
    metadata_address: Pubkey,
    conn: &Connection,
) -> Result<Option<MetadataEdition<'static>>> {
    let (address, _bump) = find_edition(metadata_address);
    let address = address.to_string();

    // TODO: This could probably all be one query if I were clever or Diesel had
    //       full outer joins.
    let editions: Vec<Edition> = editions::table
        .filter(editions::address.eq(&address))
        .limit(1)
        .load(conn)
        .context("Failed to load editions")?;

    if let Some(edition) = editions.into_iter().next() {
        return Ok(Some(MetadataEdition::Edition(edition)));
    }

    let master_editions: Vec<MasterEdition> = master_editions::table
        .filter(master_editions::address.eq(&address))
        .limit(1)
        .load(conn)
        .context("Failed to load master editions")?;

    if let Some(master_edition) = master_editions.into_iter().next() {
        return Ok(Some(MetadataEdition::MasterEdition(master_edition)));
    }

    Ok(None)
}
