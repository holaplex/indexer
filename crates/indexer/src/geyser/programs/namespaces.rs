use anchor_lang_v0_20::AccountDeserialize;
use indexer_core::prelude::*;
use namespaces::state::{Entry, ENTRY_SIZE};

use super::{accounts::namespace, AccountUpdate, Client};

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if update.data.len() == ENTRY_SIZE {
        let entry: Entry = Entry::try_deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize cardinal entry")?;

        namespace::process(client, update.key, update.slot, update.write_version, entry).await?;
    }

    Ok(())
}
