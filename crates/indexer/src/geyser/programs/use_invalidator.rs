use anchor_lang_v0_22_1::{solana_program::hash::hash, AccountDeserialize};
use cardinal_use_invalidator::state::UseInvalidator;

use super::{accounts::use_invalidator, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let use_invalidator_discriminator: &[u8] =
        &hash("account:UseInvalidator".as_bytes()).to_bytes()[..8];
    let account_discriminator = &update.data[..8];
    if account_discriminator == use_invalidator_discriminator {
        let use_invalidator: UseInvalidator =
            UseInvalidator::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize use_invalidator")?;
        use_invalidator::process(client, update.key, use_invalidator).await?
    }
    Ok(())
}
