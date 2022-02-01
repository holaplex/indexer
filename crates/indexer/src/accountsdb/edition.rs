use indexer_core::{
    db::{
        insert_into,
        models::{Edition, MasterEdition},
        tables::{editions, master_editions},
    },
    pubkeys::find_edition,
};
use metaplex_token_metadata::state::{
    Edition as EditionAccount, MasterEdition as MasterEditionTrait,
    MasterEditionV2 as MasterEditionV2Account,
};

use super::EditionKeys;
use crate::{prelude::*, util, util::MasterEdition as MasterEditionAccount, Client};

pub(super) async fn process_edition(
    client: &Client,
    edition_key: Pubkey,
    edition: &EditionAccount,
) -> Result<()> {
    let row = Edition {
        address: Owned(bs58::encode(edition_key).into_string()),
        parent_address: Owned(bs58::encode(edition.parent).into_string()),
        edition: edition
            .edition
            .try_into()
            .context("Edition ID is too high to store")?,
    };

    client
        .db(move |db| {
            insert_into(editions::table)
                .values(&row)
                .on_conflict(editions::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert edition")?;

    Ok(())
}

pub(super) async fn process_master(
    client: &Client,
    master_key: Pubkey,
    master_edition: &MasterEditionV2Account,
) -> Result<()> {
    let row = MasterEdition {
        address: Owned(bs58::encode(master_key).into_string()),
        supply: master_edition
            .supply()
            .try_into()
            .context("Master edition supply is too high to store")?,
        max_supply: master_edition
            .max_supply()
            .map(|s| {
                s.try_into()
                    .context("Master edition max supply is too high to store")
            })
            .transpose()?,
    };

    client
        .db(move |db| {
            insert_into(master_editions::table)
                .values(&row)
                .on_conflict(master_editions::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert master edition")?;

    Ok(())
}
