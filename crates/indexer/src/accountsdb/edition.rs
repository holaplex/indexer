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
};

use super::EditionKeys;
use crate::{prelude::*, util, util::MasterEdition as MasterEditionAccount, Client};

pub(super) async fn process(client: &Client, keys: EditionKeys) -> Result<()> {
    enum Type {
        Edition(EditionAccount),
        Master(MasterEditionAccount),
    }

    let (edition_key, _bump) = find_edition(keys.mint);

    let acct = client
        .get_account_opt(&edition_key)
        .context("Failed to get item edition")?;

    let mut acct = if let Some(acct) = acct {
        acct
    } else {
        debug!("No edition data found for mint {:?}", keys.mint);

        return Ok(());
    };

    let info = util::account_as_info(&edition_key, false, false, &mut acct);

    match EditionAccount::from_account_info(&info)
        .context("Failed to parse Edition")
        .map(Type::Edition)
        .or_else(|e| {
            debug!("Failed to parse Edition: {:?}", e);

            MasterEditionAccount::from_account_info(&info)
                .context("Failed to parse MasterEdition")
                .map(Type::Master)
        })? {
        Type::Edition(e) => process_edition(client, edition_key, &keys, &e).await,
        Type::Master(m) => process_master(client, edition_key, &keys, &m).await,
    }
}

async fn process_edition(
    client: &Client,
    edition_key: Pubkey,
    keys: &EditionKeys,
    edition: &EditionAccount,
) -> Result<()> {
    let row = Edition {
        address: Owned(bs58::encode(edition_key).into_string()),
        parent_address: Owned(bs58::encode(edition.parent).into_string()),
        edition: edition
            .edition
            .try_into()
            .context("Edition ID is too high to store")?,
        metadata_address: Owned(bs58::encode(keys.metadata).into_string()),
    };

    let mut acct = client
        .get_account(&edition.parent)
        .context("Failed to get item master edition")?;

    let master_edition = MasterEditionAccount::from_account_info(&util::account_as_info(
        &edition.parent,
        false,
        false,
        &mut acct,
    ))
    .context("Failed to parse edition's parent MasterEdition")?;

    process_master(client, edition.parent, keys, &master_edition).await?;

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

async fn process_master(
    client: &Client,
    master_key: Pubkey,
    keys: &EditionKeys,
    master_edition: &MasterEditionAccount,
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
        metadata_address: Owned(bs58::encode(keys.metadata).into_string()),
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
