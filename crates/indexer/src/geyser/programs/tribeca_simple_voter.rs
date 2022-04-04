use anchor_lang_v0_22_1::AccountDeserialize;
use tribeca_simple_voter::{Electorate, TokenRecord};

use super::{accounts::simple_voter, AccountUpdate, Client};
use crate::prelude::*;

const ELECTORATE_SIZE: usize = 1 + 32 + 32 + 32 + 8;
const TOKEN_RECORD: usize = 1 + 32 + 32 + 32 + 8 + 8;

async fn process_electorate(client: &Client, update: AccountUpdate) -> Result<()> {
    let electorate = Electorate::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize electorate account!")?;

    simple_voter::process_electorate(client, update.key, electorate).await
}

async fn process_token_record(client: &Client, update: AccountUpdate) -> Result<()> {
    let token_record = TokenRecord::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize token record account!")?;

    simple_voter::process_token_record(client, update.key, token_record).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        ELECTORATE_SIZE => process_electorate(client, update).await,
        TOKEN_RECORD => process_token_record(client, update).await,
        _ => Ok(()),
    }
}
