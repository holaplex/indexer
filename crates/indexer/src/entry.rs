use std::{collections::BTreeSet, mem, path::PathBuf, str::FromStr, sync::Arc};

use clap::Parser;
use indexer_core::db;
use topograph::{graph, graph::AdoptableDependents, threaded};

use crate::{
    bits::{
        auction, auction_cache, bidder_metadata, edition, get_storefronts, metadata, store_owner,
    },
    client::Client,
    prelude::*,
    util,
};

/// Convenience record for passing data to a [`ListingMetadata`] job
#[derive(Debug, Clone, Copy)]
pub struct ListingMetadata {
    /// The public key of the auction
    pub listing: Pubkey,
    /// The public key of the auction item
    pub metadata: Pubkey,
    /// The index of the item in the auction's config
    pub index: usize,
}

/// Identifying information about an auction cache account
#[derive(Debug, Clone, Copy)]
pub struct AuctionCacheKeys {
    /// The auction cache account key
    pub cache: Pubkey,
    /// The store owner's wallet address
    pub store_owner: Pubkey,
}

/// Identifying information about an auction from its cache account
#[derive(Debug, Clone, Copy)]
pub struct AuctionKeys {
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
pub type RcAuctionKeys = std::sync::Arc<AuctionKeys>;

/// The keys required to locate and associate a metadata's edition
#[derive(Debug, Clone, Copy)]
pub struct EditionKeys {
    /// The `Metadata` account pubkey
    pub metadata: Pubkey,
    /// The item's mint pubkey
    pub mint: Pubkey,
}

/// A job to be run on the process thread pool
#[derive(Debug, Clone)]
pub enum Job {
    /// Fetch the storefront list from the Holaplex API
    GetStorefronts,
    /// Fetch all bidder metadata accounts
    GetBidderMetadata,
    /// Get bidder metadata with the expectation no other jobs will use it
    ///
    /// In this case, the job itself should attempt to optimistically insert
    /// bids into the database where possible.
    GetBidderMetadataSolo,
    /// Process data for a store owner pubkey
    StoreOwner(Pubkey),
    /// Process data for an auction cache pubkey
    AuctionCache(AuctionCacheKeys),
    /// Process the join record for a listing item
    ListingMetadata(ListingMetadata),
    /// Process data for an individual item
    Metadata(Pubkey),
    /// Locate and process the edition for a token mint
    EditionForMint(EditionKeys),
    /// Process data for an auction
    Auction(RcAuctionKeys),
    /// Attempt to store bids for an auction without indexing the auction
    SoloBidsForAuction(Pubkey, bidder_metadata::BidList),
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

type ThreadPool = graph::Scheduler<Job, threaded::Executor<graph::Job<Job>>>;
/// Handle for scheduling jobs on the thread pool
pub type ThreadPoolHandle<'a> = graph::Handle<threaded::Handle<'a, graph::Job<Job>>>;

#[derive(Parser)]
struct Opts {
    /// Load a predefined list of storefront owner keys, rather than fetching
    /// from Holaplex.  Provided file should be a JSON array of strings.
    #[clap(long)]
    store_list: Option<PathBuf>,

    /// The number of threads to use.  Defaults to available core count.
    #[clap(short = 'j')]
    thread_count: Option<usize>,

    /// A comma-separated list of the root (entry) jobs to run.
    ///
    /// Valid values are 'stores' and 'bids'.  If not specified all root jobs
    /// are run.
    #[clap(short, long = "entry")]
    entries: Option<Vec<Entry>>,
}

fn create_pool(
    thread_count: Option<usize>,
    client: Arc<Client>,
    bid_map: bidder_metadata::BidMap,
    bid_dependents: &graph::RcAdoptableDependents<Job>,
) -> Result<ThreadPool> {
    let bid_dependents = bid_dependents.clone();

    let scheduler = threaded::Builder::default()
        .num_threads(thread_count)
        .build_graph(move |mut job, handle| {
            trace!("{:?}", job);

            let res = match job {
                Job::GetStorefronts => get_storefronts::run(&client, handle),
                Job::GetBidderMetadata => bidder_metadata::get(&client, &bid_map, handle),
                Job::GetBidderMetadataSolo => bidder_metadata::get_solo(&client, handle),
                Job::StoreOwner(owner) => store_owner::process(&client, owner, handle),
                Job::AuctionCache(store) => {
                    auction_cache::process(&client, store, handle, &bid_dependents)
                },
                Job::ListingMetadata(lm) => {
                    auction_cache::process_listing_metadata(&client, lm, handle)
                },
                Job::Metadata(meta) => metadata::process(&client, meta, handle),
                Job::EditionForMint(keys) => edition::process(&client, keys, handle),
                Job::Auction(ref keys) => auction::process(&client, keys, &bid_map, handle),
                Job::SoloBidsForAuction(key, ref mut bids) => {
                    auction::process_solo_bids(&client, key, mem::take(bids), handle)
                },
            };

            res.map_err(|e| error!("Job {:?} failed: {:?}", job, e))
        })
        .context("Failed to initialize thread pool")?;

    info!(
        "Created thread scheduler with {} thread(s)",
        scheduler.num_threads()
    );

    Ok(scheduler)
}

fn dispatch_entry_jobs(
    entries: Option<Vec<Entry>>,
    mut store_list: Option<PathBuf>,
    pool: &ThreadPool,
    bid_dependents: &graph::RcAdoptableDependents<Job>,
) -> Result<()> {
    let entries = {
        let mut entries: BTreeSet<_> = entries
            .unwrap_or_else(|| vec![Entry::GetStorefronts, Entry::GetBidderMetadata])
            .into_iter()
            .collect();

        if store_list.is_some() {
            entries.insert(Entry::GetStorefronts);
        }

        entries
    };

    let mut bids_run_solo = false;

    if entries.contains(&Entry::GetBidderMetadata) {
        if !entries.contains(&Entry::GetStorefronts) {
            bids_run_solo = true;
        }
    } else {
        bid_dependents.lock().complete(pool).unwrap();
    }

    for entry in entries {
        match entry {
            Entry::GetStorefronts => {
                if let Some(store_list) = store_list.take() {
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
            },
            Entry::GetBidderMetadata if bids_run_solo => {
                pool.push(Job::GetBidderMetadataSolo);
                bid_dependents.lock().abandon().unwrap();
            },
            Entry::GetBidderMetadata => {
                let dependents = pool.push_dependency(Job::GetBidderMetadata, None);
                bid_dependents.lock().adopt(pool, dependents).unwrap();
            },
        }
    }

    Ok(())
}

pub fn run() -> Result<()> {
    let Opts {
        thread_count,
        store_list,
        entries,
    } = Opts::parse();

    let db = db::connect(db::ConnectMode::Write).context("Failed to connect to Postgres")?;

    let bid_dependents = AdoptableDependents::new().rc();
    let pool = create_pool(
        thread_count,
        Client::new_rc(db).context("Failed to construct Client")?,
        bidder_metadata::BidMap::default(),
        &bid_dependents,
    )?;

    // ---- BEGIN INDEXER RUN ----
    let start_time = Local::now();
    dispatch_entry_jobs(entries, store_list, &pool, &bid_dependents)?;
    pool.join();
    let end_time = Local::now();
    // ---- END INDEXER RUN ----

    info!(
        "Indexer run finished in {}",
        util::duration_hhmmssfff(end_time - start_time)
    );

    Ok(())
}
