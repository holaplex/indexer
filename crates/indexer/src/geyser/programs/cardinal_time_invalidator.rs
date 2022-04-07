use ::cardinal_time_invalidator::state::TimeInvalidator;
use anchor_lang_v0_22_1::{solana_program::hash::hash, AccountDeserialize};

use super::{accounts::cardinal_time_invalidator, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let time_invalidator_discriminator: &[u8] =
        &hash("account:TimeInvalidator".as_bytes()).to_bytes()[..8];
    let account_discriminator = &update.data[..8];
    if account_discriminator == time_invalidator_discriminator {
        let time_invalidator: TimeInvalidator =
            TimeInvalidator::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize time_invalidator")?;
        cardinal_time_invalidator::process(client, update.key, time_invalidator).await?
    }
    Ok(())
}
