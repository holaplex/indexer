use anchor_lang_v0_22_1::AccountDeserialize;
use cardinal_token_manager::state::TokenManager;

use super::{accounts::token_manager, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let token_manager: TokenManager = TokenManager::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize token_manager")?;
    token_manager::process(client, update.key, token_manager).await
    // let token_manager_discriminator: &[u8] =
    //     hash("account:TokenManager".as_bytes()).to_bytes()[..8];
    // let account_discriminator = &update.data[..8];
    // match account_discriminator {
    //     token_manager_discriminator => {
    //         let token_manager: TokenManager =
    //             TokenManager::try_deserialize(&mut update.data.as_slice())
    //                 .context("Failed to deserialize candy_machine")?;
    //         token_manager::process(client, update.key, token_manager).await
    //     },
    //     _ => Ok(()),
    // }
}
