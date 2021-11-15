//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::path::PathBuf;

use baby_pool::ThreadPool;
use clap::Parser;
use indexer_core::prelude::*;
use solana_sdk::pubkey::Pubkey;

mod auction;
mod auction_cache;
mod client;
mod get_storefronts;
mod metadata;
mod store_owner;
pub mod util;

pub use client::Client;

/// The three pubkeys associated with a single Metaplex auction
#[derive(Debug, Clone, Copy)]
pub struct AuctionKeys {
    auction: Pubkey,
    manager: Pubkey,
    vault: Pubkey,
}

/// Convenience alias for a shared `AuctionKeys`
pub type RcAuctionKeys = std::sync::Arc<AuctionKeys>;

/// A job to be run on the process thread pool
#[derive(Debug, Clone)]
pub enum Job {
    /// Fetch the storefront list from the Holaplex API
    GetStorefronts,
    /// Process data for a store owner pubkey
    StoreOwner(Pubkey),
    /// Process data for an auction cache pubkey
    AuctionCache(Pubkey),
    /// Process data for an individual item
    Metadata(Pubkey),
    /// Process data for an auction
    Auction(RcAuctionKeys),
}

type ThreadPoolHandle<'a> = baby_pool::ThreadPoolHandle<'a, Job>;

#[derive(Parser)]
struct Opts {
    /// Load a predefined list of storefront owner keys, rather than fetching
    /// from Holaplex.  Provided file should be a JSON array of strings.
    #[clap(long)]
    store_list: Option<PathBuf>,

    /// The number of threads to use.  Defaults to available core count.
    #[clap(short = 'j')]
    thread_count: Option<usize>,
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            thread_count,
            store_list,
        } = Opts::parse();

        let client = Client::new_rc().context("Failed to construct Client")?;

        let pool = ThreadPool::new(thread_count, move |job, handle| {
            trace!("{:?}", job);

            let res = match job {
                Job::GetStorefronts => get_storefronts::run(&handle),
                Job::StoreOwner(owner) => store_owner::process(&client, owner, &handle),
                Job::AuctionCache(store) => auction_cache::process(&client, store, &handle),
                Job::Metadata(meta) => metadata::process(&client, meta, &handle),
                Job::Auction(ref keys) => auction::process(&client, keys, &handle),
            };

            match res {
                Ok(()) => (),
                Err(e) => error!("Job {:?} failed: {:?}", job, e),
            }
        });

        if let Some(store_list) = store_list {
            let list: Vec<String> = serde_json::from_reader(
                std::fs::File::open(store_list).context("Couldn't open storefront list")?,
            )
            .context("Couldn't parse storefront list")?;

            list.into_iter()
                .filter_map(|i| {
                    Pubkey::try_from(&*i)
                        .map_err(|e| error!("Failed to parse pubkey: {:?}", e))
                        .ok()
                })
                .for_each(|k| pool.push(Job::StoreOwner(k)));
        } else {
            pool.push(Job::GetStorefronts);
        }

        pool.join();

        Ok(())
    });
}
