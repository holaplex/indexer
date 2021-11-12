//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

mod get_storefronts;

use std::env;

use baby_pool::ThreadPool;
use indexer_core::{
    prelude::*,
    pubkeys::{find_store_address, find_store_indexer},
};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::RpcClient,
    rpc_request::RpcError,
};
use solana_sdk::pubkey::Pubkey;

/// A job to be run on the process thread pool
#[derive(Debug, Clone, Copy)]
pub enum Job {
    /// Fetch the storefront list from the Holaplex API
    GetStorefronts,
    /// Process data for a store owner pubkey
    StoreOwner(Pubkey),
    /// Process data for an auction cache pubkey
    AuctionCache(Pubkey),
}

fn main() {
    indexer_core::run(|| {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);
        let _client = RpcClient::new(endpoint);

        let pool = ThreadPool::new(None, |job, handle| {
            let res = match job {
                Job::GetStorefronts => get_storefronts::run(handle),
                Job::StoreOwner(_) => todo!("store_owner"),
                Job::AuctionCache(_) => todo!("auction_cache"),
            };

            match res {
                Ok(()) => (),
                Err(e) => error!("Job {:?} failed: {:?}", job, e),
            }
        });

        pool.push(Job::GetStorefronts);
        pool.join();

        Ok(())
    });
}

fn _main_old() {
    indexer_core::run(|| {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);
        let client = RpcClient::new(endpoint);

        let mut indexed_stores = Vec::new();
        let mut s = String::new();

        while let Ok(line) = std::io::stdin()
            .read_line(&mut s)
            .map_err(|e| warn!("{:?}", e))
        {
            if line == 0 {
                break;
            }

            if let Ok(owner) = Pubkey::try_from(s.trim()).map_err(|e| {
                error!(
                    "{:?}",
                    Error::from(e).context(format!("Failed to parse pubkey {:?}", s))
                );
            }) {
                let (store, _bump) = find_store_address(owner);

                let mut pages = (0..)
                    .map_while(|i| {
                        let (indexer, _bump) = find_store_indexer(store, i);

                        match client.get_account(&indexer) {
                            Ok(acct) => Some((acct, i)),
                            Err(ClientError {
                                kind: ClientErrorKind::RpcError(RpcError::ForUser(msg)),
                                ..
                            }) if msg.starts_with("AccountNotFound:") => None,
                            Err(e) => {
                                error!(
                                    "{:?}",
                                    Error::from(e).context(format!("Failed to get page {}", i))
                                );
                                None
                            },
                        }
                    })
                    .peekable();

                if pages.peek().is_some() {
                    info!("Indexer found for store {:?} (owner {:?})", store, owner);
                    indexed_stores.push(owner);
                } else {
                    warn!("No indexer for store {:?} (owner {:?})", store, owner);
                }

                pages.for_each(|(acct, i)| info!("Indexer, page {}: {:?}", i, acct));
            }

            s.clear();
        }

        info!("Found {} indexed store(s).", indexed_stores.len());

        for key in indexed_stores {
            println!("{:?}", key);
        }

        Ok(())
    });
}
