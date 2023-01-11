use indexer::prelude::*;
use indexer_core::db::{
    insert_into,
    models::{Edition, MasterEdition},
    tables::{editions, master_editions},
};
use mpl_token_metadata::state::{
    Edition as EditionAccount, MasterEdition as MasterEditionTrait,
    MasterEditionV2 as MasterEditionV2Account,
};

use super::Client;

pub(crate) async fn process(
    client: &Client,
    edition_key: Pubkey,
    edition: EditionAccount,
    slot: u64,
) -> Result<()> {
    let row = Edition {
        address: Owned(bs58::encode(edition_key).into_string()),
        parent_address: Owned(bs58::encode(edition.parent).into_string()),
        edition: edition
            .edition
            .try_into()
            .context("Edition ID is too high to store")?,
        slot: Some(
            slot.try_into()
                .context("Edition slot was too big to store")?,
        ),
    };

    client
        .db()
        .run(move |db| {
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

pub(crate) async fn process_master(
    client: &Client,
    master_key: Pubkey,
    master_edition: MasterEditionV2Account,
    slot: u64,
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
        slot: Some(
            slot.try_into()
                .context("Master edition slot was too big to store")?,
        ),
    };

    client
        .db()
        .run(move |db| {
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
