use solana_program::program_pack::Pack;
use spl_token::{instruction::TokenInstruction, state::Account as TokenAccount};

use super::{accounts::token, instructions::token as token_instruction, AccountUpdate, Client};
use crate::prelude::*;

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
    let ins = TokenInstruction::unpack(data).context("Invalid spl token instruction")?;

    match ins {
        TokenInstruction::Burn { .. } => {
            token_instruction::process_burn_instruction(client, accounts, slot).await
        },
        _ => Ok(()),
    }
}
