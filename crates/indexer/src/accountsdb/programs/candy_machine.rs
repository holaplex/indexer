use anchor_lang_v0_21_0::{AccountDeserialize, AnchorDeserialize};
use mpl_candy_machine::{
    CandyMachine, CollectionPDA, ConfigLine, CONFIG_ARRAY_START, CONFIG_LINE_SIZE,
};

use super::{accounts::candy_machine, AccountUpdate, Client};
use crate::prelude::*;

const COLLECTION_PDA_SIZE: usize = 8 + 64;

pub async fn process_collection_pda(client: &Client, update: AccountUpdate) -> Result<()> {
    let collection_pda: CollectionPDA = CollectionPDA::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize collection pda")?;

    candy_machine::process_collection_pda(client, update.key, collection_pda).await
}

pub async fn process_config_line(client: &Client, update: AccountUpdate) -> Result<()> {
    let config_line: ConfigLine = ConfigLine::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize config line")?;

    candy_machine::process_config_line(client, update.key, config_line).await
}

pub async fn process_cm(client: &Client, update: AccountUpdate) -> Result<()> {
    let candy_machine: CandyMachine = CandyMachine::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize candy_machine")?;

    candy_machine::process(client, update.key, candy_machine).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        COLLECTION_PDA_SIZE => process_collection_pda(client, update).await,
        CONFIG_LINE_SIZE => process_config_line(client, update).await,
        CONFIG_ARRAY_START => process_cm(client, update).await,
        _ => Ok(()),
    }
}
