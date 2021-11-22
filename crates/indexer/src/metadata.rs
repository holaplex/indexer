use indexer_core::prelude::*;
use metaplex_token_metadata::state::Metadata;
use solana_sdk::pubkey::Pubkey;

use crate::{util, Client, ThreadPoolHandle};

pub fn process(client: &Client, meta: Pubkey, _handle: &ThreadPoolHandle) -> Result<()> {
    let mut acct = client
        .get_account(&meta)
        .context("Failed to get item metadata")?;

    let meta = Metadata::from_account_info(&util::account_as_info(&meta, false, false, &mut acct))
        .context("Failed to parse Metadata")?;

    // TODO
    info!("NFT! {:?}", meta);

    Ok(())
}
