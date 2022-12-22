use ::cardinal_time_invalidator::state::TimeInvalidator;
use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use indexer::prelude::*;

use super::{accounts::cardinal_time_invalidator, AccountUpdate, Client};

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let account_discriminator = &update.data[..8];

    if account_discriminator == TimeInvalidator::discriminator() {
        let time_invalidator: TimeInvalidator =
            TimeInvalidator::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize time_invalidator")?;

        cardinal_time_invalidator::process(client, update.key, time_invalidator).await?;
    }

    Ok(())
}
