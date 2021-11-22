use indexer_core::{
    prelude::*,
    pubkeys::{find_store_address, find_store_indexer},
};
use metaplex::state::StoreIndexer;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::{util, Client, Job, ThreadPoolHandle};

// TODO: reroll the loop into multiple jobs
pub fn process(client: &Client, owner: Pubkey, handle: &ThreadPoolHandle) -> Result<()> {
    let (store, _bump) = find_store_address(owner);

    for (key, acct) in (0..).map_while(|i| {
        let (indexer, _bump) = find_store_indexer(store, i);

        client
            .get_account_opt(&indexer)
            .map_err(|e| error!("Failed to get store indexer: {:?}", e))
            .ok()
            .flatten()
            .map(|a| (indexer, a))
    }) {
        process_page(&key, acct, handle)
            .map_err(|e| error!("Failed to process store indexer: {:?}", e))
            .ok();
    }

    Ok(())
}

fn process_page(key: &Pubkey, mut acct: Account, handle: &ThreadPoolHandle) -> Result<()> {
    let indexer =
        StoreIndexer::from_account_info(&util::account_as_info(key, false, false, &mut acct))
            .context("Failed to parse StoreIndexer")?;

    for cache in indexer.auction_caches {
        handle.push(Job::AuctionCache(cache));
    }

    Ok(())
}
