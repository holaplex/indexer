#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]

use std::env;

use indexer_core::{
    prelude::*,
    pubkeys::{find_store_address, find_store_indexer},
    ThreadPool,
};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::RpcClient,
    rpc_request::RpcError,
};
use solana_sdk::pubkey::Pubkey;

enum Job {
    GetStorefronts,
    StoreOwner(Pubkey),
    AuctionCache(Pubkey),
}

fn main() {
    indexer_core::run(|| {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);
        let client = RpcClient::new(endpoint);

        let pool = ThreadPool::new(None, |job, handle| match job {
            Job::GetStorefronts => {
                handle.push(Job::GetStorefronts);
                todo!()
            },
            Job::StoreOwner(k) => todo!("store_owner"),
            Job::AuctionCache(k) => todo!("auction_cache"),
        });

        std::thread::sleep(std::time::Duration::from_secs(1));

        pool.push(Job::GetStorefronts);
        pool.join();

        Ok(())
    });
}

fn main_old() {
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
