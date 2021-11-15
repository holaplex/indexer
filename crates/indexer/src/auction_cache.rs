use std::sync::Arc;

use indexer_core::prelude::*;
use metaplex::state::AuctionCache;
use solana_sdk::pubkey::Pubkey;

use crate::{util, AuctionKeys, Client, Job, ThreadPoolHandle};

pub fn process(client: &Client, cache: Pubkey, handle: &ThreadPoolHandle) -> Result<()> {
    let mut acct = client
        .get_account(&cache)
        .context("Failed to get auction cache")?;

    let cache =
        AuctionCache::from_account_info(&util::account_as_info(&cache, false, false, &mut acct))
            .context("Failed to parse AuctionCache")?;

    let AuctionCache {
        metadata,
        auction,
        vault,
        auction_manager: manager,
        ..
    } = cache;

    for meta in metadata {
        handle.push(Job::Metadata(meta));
    }

    handle.push(Job::Auction(Arc::new(AuctionKeys {
        auction,
        vault,
        manager,
    })));

    Ok(())
}
