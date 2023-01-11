use ::cardinal_use_invalidator::state::UseInvalidator;
use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use indexer::prelude::*;

use super::{accounts::cardinal_use_invalidator, AccountUpdate, Client};

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let account_discriminator = &update.data[..8];

    if account_discriminator == UseInvalidator::discriminator() {
        let use_invalidator: UseInvalidator =
            UseInvalidator::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize use_invalidator")?;

        cardinal_use_invalidator::process(client, update.key, use_invalidator).await?;
    }

    Ok(())
}
