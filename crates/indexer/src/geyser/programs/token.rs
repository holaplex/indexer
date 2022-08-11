use solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

use super::{accounts::token, instructions::token as token_instruction, AccountUpdate, Client};
use crate::prelude::*;

const BURN: u8 = 8;
async fn process_token(client: &Client, update: AccountUpdate) -> Result<()> {
    let token_account = TokenAccount::unpack_unchecked(&update.data)
        .context("Failed to deserialize token account data!")?;
    token::process(client, update.key, token_account, update.slot).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if update.data.len() != TokenAccount::LEN {
        return Ok(());
    }
    process_token(client, update).await
}

pub(crate) async fn process_instruction(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let (&discriminator, _) = data
        .split_first()
        .context("invalid spl token instruction")?;

    match discriminator {
        BURN => token_instruction::process_burn_instruction(client, accounts, slot).await,
        _ => Ok(()),
    }
}
