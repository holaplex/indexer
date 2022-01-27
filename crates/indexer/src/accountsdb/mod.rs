mod auction;
mod auction_cache;
mod bidder_metadata;
mod edition;
mod get_storefronts;
mod metadata;
mod metadata_uri;
mod store_owner;
mod token_account;

use std::{path::PathBuf, str::FromStr};

use indexer_core::{clap, clap::Parser, pubkeys};
use indexer_rabbitmq::accountsdb::Message;

use crate::{client::Client, prelude::*};

pub fn process_message(msg: Message, client: &Client) -> Result<()> {
    match msg {
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::metadata() => {
            metadata::process(client, key, data)
        },
        Message::AccountUpdate { .. } | Message::InstructionNotify { .. } => Ok(()),
    }
}

/// Convenience record for passing data to a [`ListingMetadata`] job
#[derive(Debug, Clone, Copy)]
struct ListingMetadata {
    /// The public key of the auction
    pub listing: Pubkey,
    /// The public key of the auction item
    pub metadata: Pubkey,
    /// The index of the item in the auction's config
    pub index: usize,
}

/// Identifying information about an auction cache account
#[derive(Debug, Clone, Copy)]
struct AuctionCacheKeys {
    /// The auction cache account key
    pub cache: Pubkey,
    /// The store owner's wallet address
    pub store_owner: Pubkey,
}

/// Identifying information about an auction from its cache account
#[derive(Debug, Clone, Copy)]
struct AuctionKeys {
    /// The `AuctionData` account pubkey
    pub auction: Pubkey,
    /// The auction's vault pubkey
    pub vault: Pubkey,
    /// The store owner's wallet address
    pub store_owner: Pubkey,
    /// The timestamp the auction was recorded at
    pub created_at: chrono::NaiveDateTime,
}

/// Convenience alias for a shared `AuctionKeys`
type RcAuctionKeys = std::sync::Arc<AuctionKeys>;

/// The keys required to locate and associate a metadata's edition
#[derive(Debug, Clone, Copy)]
struct EditionKeys {
    /// The `Metadata` account pubkey
    pub metadata: Pubkey,
    /// The item's mint pubkey
    pub mint: Pubkey,
}

/// A job to be run on the process thread pool
#[derive(Debug, Clone)]
enum Job {
    // /// Fetch the storefront list from the Holaplex API
// GetStorefronts,
// /// Fetch all bidder metadata accounts
// GetBidderMetadata,
// /// Get bidder metadata with the expectation no other jobs will use it
// ///
// /// In this case, the job itself should attempt to optimistically insert
// /// bids into the database where possible.
// GetBidderMetadataSolo,
// /// Process data for a store owner pubkey
// StoreOwner(Pubkey),
// /// Process data for an auction cache pubkey
// AuctionCache(AuctionCacheKeys),
// /// Process the join record for a listing item
// ListingMetadata(ListingMetadata),
// /// Process data for an individual item
// MetadataUri(Pubkey, String),
// /// Locate and process the edition for a token mint
// EditionForMint(EditionKeys),
// /// Process data for an auction
// Auction(RcAuctionKeys),
// /// Attempt to store bids for an auction without indexing the auction
// SoloBidsForAuction(Pubkey, bidder_metadata::BidList),
// /// Index token accounts so we can know who holds what NFTs
// TokenAccount(Pubkey, TokenAccount),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Entry {
    GetStorefronts,
    GetBidderMetadata,
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "storefromts" | "stores" => Self::GetStorefronts,
            "bidder-metadata" | "bids" => Self::GetBidderMetadata,
            s => bail!("Unexpected entrypoint {:?}", s),
        })
    }
}

#[derive(Debug, Parser)]
struct Opts {
    /// A valid base URL to use when fetching IPFS links
    #[clap(long, env)]
    ipfs_cdn: Option<String>,

    /// A valid base URL to use when fetching Arweave links
    #[clap(long, env)]
    arweave_cdn: Option<String>,

    // Arguments passed to `create_pool`
    #[clap(flatten)]
    pool_cfg: ThreadPoolConfig,

    // Arguments passed to `dispatch_entry_jobs`
    #[clap(flatten)]
    entry_cfg: EntryConfig,
}

#[derive(Debug, Parser)]
struct ThreadPoolConfig {
    /// The number of threads to use.  Defaults to available core count.
    #[clap(short = 'j')]
    thread_count: Option<usize>,

    /// Pull off-chain JSON data for metadata accounts
    #[clap(long, parse(from_flag), env)]
    metadata_json: bool,
}

#[derive(Debug, Parser)]
struct EntryConfig {
    /// A comma-separated list of the root (entry) jobs to run.
    ///
    /// Valid values are 'stores' and 'bids'.  If not specified all root jobs
    /// are run.
    #[clap(short, long = "entry")]
    entries: Option<Vec<Entry>>,

    /// Load a predefined list of storefront owner keys, rather than fetching
    /// from Holaplex.  Provided file should be a JSON array of strings.
    #[clap(long)]
    store_list: Option<PathBuf>,
}

// #[allow(clippy::needless_pass_by_value)]
// fn create_pool(
//     cfg: ThreadPoolConfig,
//     client: Arc<Client>,
//     bid_map: bidder_metadata::BidMap,
//     bid_dependents: &graph::RcAdoptableDependents<Job>,
// ) -> Result<ThreadPool> {
//     let ThreadPoolConfig {
//         thread_count,
//         metadata_json,
//     } = cfg;

//     if !metadata_json {
//         warn!("Skipping metadata JSON");
//     }

//     let bid_dependents = bid_dependents.clone();

//     let scheduler = threaded::Builder::default()
//         .num_threads(thread_count)
//         .build_graph(move |mut job, handle| {
//             trace!("{:?}", job);

//             let res = match job {
//                 Job::GetStorefronts => get_storefronts::run(&client, handle),
//                 Job::GetBidderMetadata => bidder_metadata::get(&client, &bid_map, handle),
//                 Job::GetBidderMetadataSolo => bidder_metadata::get_solo(&client, handle),
//                 Job::StoreOwner(owner) => store_owner::process(&client, owner, handle),
//                 Job::AuctionCache(store) => {
//                     auction_cache::process(&client, store, handle, &bid_dependents)
//                 },
//                 Job::ListingMetadata(lm) => {
//                     auction_cache::process_listing_metadata(&client, lm, handle)
//                 },
//                 Job::MetadataUri(..) if !metadata_json => Ok(()),
//                 Job::MetadataUri(meta, ref uri) => {
//                     metadata_uri::process(&client, meta, uri.clone(), handle)
//                 },
//                 Job::EditionForMint(keys) => edition::process(&client, keys, handle),
//                 Job::Auction(ref keys) => auction::process(&client, keys, &bid_map, handle),
//                 Job::SoloBidsForAuction(key, ref mut bids) => {
//                     auction::process_solo_bids(&client, key, mem::take(bids), handle)
//                 },
//                 Job::TokenAccount(ref pubkey, token_account) => {
//                     token_account::process(&client, *pubkey, token_account)
//                 },
//             };

//             res.map_err(|e| error!("Job {:?} failed: {:?}", job, e))
//         })
//         .context("Failed to initialize thread pool")?;

//     info!(
//         "Created thread scheduler with {} thread(s)",
//         scheduler.num_threads()
//     );

//     Ok(scheduler)
// }

// fn dispatch_entry_jobs(
//     cfg: EntryConfig,
//     pool: &ThreadPool,
//     bid_dependents: &graph::RcAdoptableDependents<Job>,
// ) -> Result<()> {
//     let EntryConfig {
//         entries,
//         mut store_list,
//     } = cfg;

//     let entries = {
//         let mut entries: BTreeSet<_> = entries
//             .unwrap_or_else(|| vec![Entry::GetStorefronts, Entry::GetBidderMetadata])
//             .into_iter()
//             .collect();

//         if store_list.is_some() {
//             entries.insert(Entry::GetStorefronts);
//         }

//         entries
//     };

//     let mut bids_run_solo = false;

//     if entries.contains(&Entry::GetBidderMetadata) {
//         if !entries.contains(&Entry::GetStorefronts) {
//             bids_run_solo = true;
//         }
//     } else {
//         bid_dependents.lock().complete(pool).unwrap();
//     }

//     for entry in entries {
//         match entry {
//             Entry::GetStorefronts => {
//                 if let Some(store_list) = store_list.take() {
//                     let list: Vec<String> = serde_json::from_reader(
//                         std::fs::File::open(store_list).context("Couldn't open storefront list")?,
//                     )
//                     .context("Couldn't parse storefront list")?;

//                     list.into_iter()
//                         .filter_map(|i| {
//                             Pubkey::try_from(&*i)
//                                 .map_err(|e| error!("Failed to parse pubkey: {:?}", e))
//                                 .ok()
//                         })
//                         .for_each(|k| pool.push(Job::StoreOwner(k)));
//                 } else {
//                     pool.push(Job::GetStorefronts);
//                 }
//             },
//             Entry::GetBidderMetadata if bids_run_solo => {
//                 pool.push(Job::GetBidderMetadataSolo);
//                 bid_dependents.lock().abandon().unwrap();
//             },
//             Entry::GetBidderMetadata => {
//                 let dependents = pool.push_dependency(Job::GetBidderMetadata, None);
//                 bid_dependents.lock().adopt(pool, dependents).unwrap();
//             },
//         }
//     }

//     Ok(())
// }

// fn run() -> Result<()> {
//     let opts = Opts::parse();

//     debug!("{:#?}", opts);

//     let Opts {
//         arweave_cdn,
//         ipfs_cdn,
//         pool_cfg,
//         entry_cfg,
//     } = opts;

//     let db = db::connect(db::ConnectMode::Write).context("Failed to connect to Postgres")?;

//     let bid_dependents = AdoptableDependents::new().rc();
//     let pool = create_pool(
//         pool_cfg,
//         Client::new_rc(
//             db,
//             ipfs_cdn
//                 .ok_or_else(|| anyhow!("Missing IPFS CDN"))?
//                 .parse()
//                 .context("Failed to parse IPFS CDN URL")?,
//             arweave_cdn
//                 .ok_or_else(|| anyhow!("Missing Arweave CDN"))?
//                 .parse()
//                 .context("Failed to parse Arweave CDN URL")?,
//         )
//         .context("Failed to construct Client")?,
//         bidder_metadata::BidMap::default(),
//         &bid_dependents,
//     )?;

//     // ---- BEGIN INDEXER RUN ----
//     let start_time = Local::now();
//     dispatch_entry_jobs(entry_cfg, &pool, &bid_dependents)?;
//     pool.join();
//     let end_time = Local::now();
//     // ---- END INDEXER RUN ----

//     info!(
//         "Indexer run finished in {}",
//         util::duration_hhmmssfff(end_time - start_time)
//     );

//     Ok(())
// }
