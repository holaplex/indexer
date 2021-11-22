//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{env, path::PathBuf};

use clap::Parser;
use indexer_core::db;
use prelude::*;
use solana_sdk::pubkey::Pubkey;
use topograph::{graph, threaded};

mod prelude {
    pub use indexer_core::prelude::*;
    pub use topograph::prelude::*;
}

mod auction;
mod auction_cache;
mod client;
mod edition;
mod get_storefronts;
mod metadata;
mod store_owner;
pub mod util;

pub use client::Client;

/// Convenience record for passing data to a [`ListingMetadata`] job
#[derive(Debug, Clone, Copy)]
pub struct ListingMetadata {
    listing: Pubkey,
    metadata: Pubkey,
}

/// The three pubkeys associated with a single Metaplex auction
#[derive(Debug, Clone, Copy)]
pub struct AuctionKeys {
    auction: Pubkey,
    manager: Pubkey,
    vault: Pubkey,
    store: Pubkey,
    created_at: chrono::NaiveDateTime,
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
    /// Process the join record for a listing item
    ListingMetadata(ListingMetadata),
    /// Process data for an individual item
    Metadata(Pubkey),
    /// Locate and process the edition for a token mint
    EditionForMint(Pubkey),
    /// Process data for an auction
    Auction(RcAuctionKeys),
}

type ThreadPoolHandle<'a> = graph::Handle<threaded::Handle<'a, graph::Job<Job>>>;

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

        let db = db::connect(
            env::var_os("DATABASE_WRITE_URL")
                .or_else(|| env::var_os("DATABASE_URL"))
                .ok_or_else(|| anyhow!("No value found for DATABASE_WRITE_URL or DATABASE_URL"))
                .map(move |v| v.to_string_lossy().into_owned())?,
        )
        .context("Failed to connect to Postgres")?;
        let client = Client::new_rc(db).context("Failed to construct Client")?;

        let pool = threaded::Builder::default()
            .num_threads(thread_count)
            .build_graph(move |job, handle| {
                trace!("{:?}", job);

                let res = match job {
                    Job::GetStorefronts => get_storefronts::run(handle),
                    Job::StoreOwner(owner) => store_owner::process(&client, owner, handle),
                    Job::AuctionCache(store) => auction_cache::process(&client, store, handle),
                    Job::ListingMetadata(lm) => {
                        auction_cache::process_listing_metadata(&client, lm, handle)
                    },
                    Job::Metadata(meta) => metadata::process(&client, meta, handle),
                    Job::EditionForMint(mint) => edition::process(&client, mint, handle),
                    Job::Auction(ref keys) => auction::process(&client, keys, handle),
                };

                res.map_err(|e| error!("Job {:?} failed: {:?}", job, e))
            })
            .context("Failed to initialize thread pool")?;

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
