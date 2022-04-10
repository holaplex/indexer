use ::cardinal_token_manager::state::TokenManager;
use anchor_lang_v0_22_1::{AccountDeserialize, Discriminator};

use super::{accounts::cardinal_token_manager, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let account_discriminator = &update.data[..8];
    if account_discriminator == TokenManager::discriminator() {
        let token_manager: TokenManager =
            TokenManager::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize token_manager")?;
        cardinal_token_manager::process(client, update.key, token_manager).await?
    }
    Ok(())
}
