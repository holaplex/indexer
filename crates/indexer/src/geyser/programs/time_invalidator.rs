use anchor_lang_v0_22_1::{solana_program::hash::hash, AccountDeserialize};
use cardinal_time_invalidator::state::TimeInvalidator;

use super::{accounts::time_invalidator, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    // let time_invalidator: TimeInvalidator =
    //     TimeInvalidator::try_deserialize(&mut update.data.as_slice())?;
    // time_invalidator::process(client, update.key, time_invalidator).await
    let time_invalidator_discriminator: &[u8] =
        &hash("account:TimeInvalidator".as_bytes()).to_bytes()[..8];
    let account_discriminator = &update.data[..8];
    if account_discriminator == time_invalidator_discriminator {
        let time_invalidator: TimeInvalidator =
            TimeInvalidator::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize time_invalidator")?;
        time_invalidator::process(client, update.key, time_invalidator).await?
    }
    Ok(())
}
