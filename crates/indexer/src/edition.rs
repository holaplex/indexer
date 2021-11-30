use indexer_core::{
    db::{
        insert_into,
        models::{Edition, MasterEdition},
        tables::{
            editions::{address, editions},
            master_editions::{address as master_address, master_editions},
        },
    },
    pubkeys::find_edition,
};
use metaplex_token_metadata::state::{
    Edition as EditionAccount, MasterEdition as MasterEditionTrait,
};

use crate::{
    prelude::*, util, util::MasterEdition as MasterEditionAccount, Client, ThreadPoolHandle,
};

pub fn process(client: &Client, mint_key: Pubkey, _handle: ThreadPoolHandle) -> Result<()> {
    let (edition_key, _bump) = find_edition(mint_key);

    let mut acct = client
        .get_account(&edition_key)
        .context("Failed to get item edition")?;

    let info = util::account_as_info(&edition_key, false, false, &mut acct);

    EditionAccount::from_account_info(&info)
        .map_err(Into::into)
        .and_then(|e| process_edition(client, edition_key, &e))
        .or_else(|e| {
            debug!("Failed to parse Edition: {:?}", e);

            let master = MasterEditionAccount::from_account_info(&info)
                .context("Failed to parse MasterEdition")?;

            process_master(client, edition_key, &master)
        })
}

fn process_edition(client: &Client, edition_key: Pubkey, edition: &EditionAccount) -> Result<()> {
    let row = Edition {
        address: Owned(bs58::encode(edition_key).into_string()),
        parent_address: Owned(bs58::encode(edition.parent).into_string()),
        edition: edition
            .edition
            .try_into()
            .context("Edition ID is too high to store")?,
    };

    let db = client.db()?;

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

    process_master(client, edition.parent, &master_edition)?;

    insert_into(editions)
        .values(&row)
        .on_conflict(address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert edition")?;

    Ok(())
}

fn process_master(
    client: &Client,
    master_key: Pubkey,
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
    };

    let db = client.db()?;

    insert_into(master_editions)
        .values(&row)
        .on_conflict(master_address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert master edition")?;

    Ok(())
}
