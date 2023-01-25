use anchor_lang_v0_24::AccountDeserialize;
use indexer::prelude::*;
use mpl_candy_machine::{CandyMachine, CollectionPDA};

use super::{accounts::candy_machine, AccountUpdate, Client};

const COLLECTION_PDA_SIZE: usize = 8 + 64;

pub async fn process_collection_pda(client: &Client, update: AccountUpdate) -> Result<()> {
    let collection_pda: CollectionPDA = CollectionPDA::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize collection pda")?;

    candy_machine::process_collection_pda(client, update.key, collection_pda).await
}

pub async fn process_cm(client: &Client, update: AccountUpdate) -> Result<()> {
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize candy_machine")?;

    candy_machine::process(client, update.key, candy_machine).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        _ => process_cm(client, update).await,
    }
}
