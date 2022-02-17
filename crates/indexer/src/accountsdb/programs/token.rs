use solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

use super::{accounts::token, AccountUpdate, Client};
use crate::prelude::*;

async fn process_token(client: &Client, update: AccountUpdate) -> Result<()> {
    let token_account = TokenAccount::unpack_unchecked(&update.data)
        .context("Failed to deserialize token account data!")?;
    token::process(client, update.key, token_account).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    process_token(client, update).await
}
