use indexer_core::pubkeys::{find_store_address, find_store_indexer};
use metaplex::state::StoreIndexer;
use solana_sdk::account::Account;

use crate::{prelude::*, util, AuctionCacheKeys, Client, Job, ThreadPoolHandle};

// TODO: reroll the loop into multiple jobs
#[allow(clippy::unnecessary_wraps)]
pub fn process(client: &Client, owner: Pubkey, handle: ThreadPoolHandle) -> Result<()> {
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
        process_page(owner, &key, acct, handle)
            .map_err(|e| error!("Failed to process store indexer: {:?}", e))
            .ok();
    }

    Ok(())
}

fn process_page(
    store_owner: Pubkey,
    key: &Pubkey,
    mut acct: Account,
    handle: ThreadPoolHandle,
) -> Result<()> {
    let indexer =
        StoreIndexer::from_account_info(&util::account_as_info(key, false, false, &mut acct))
            .context("Failed to parse StoreIndexer")?;

    for cache in indexer.auction_caches {
        handle.push(Job::AuctionCache(AuctionCacheKeys { cache, store_owner }));
    }

    Ok(())
}
