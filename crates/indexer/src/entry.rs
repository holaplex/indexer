use std::{collections::BTreeSet, env, path::PathBuf, str::FromStr};

use clap::Parser;
use indexer_core::db;
use topograph::{graph, threaded};

use crate::{
    bits::{
        auction, auction_cache, bidder_metadata, edition, get_storefronts, metadata, store_owner,
    },
    client::Client,
    prelude::*,
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

/// A job to be run on the process thread pool
#[derive(Debug, Clone)]
pub enum Job {
    /// Fetch the storefront list from the Holaplex API
    GetStorefronts,
    /// Fetch all bidder metadata accounts
    GetBidderMetadata,
    /// Process data for a store owner pubkey
    StoreOwner(Pubkey),
    /// Process data for an auction cache pubkey
    AuctionCache(AuctionCacheKeys),
    /// Process the join record for a listing item
    ListingMetadata(ListingMetadata),
    /// Process data for an individual item
    Metadata(Pubkey),
    /// Locate and process the edition for a token mint
    EditionForMint(Pubkey),
    /// Process data for an auction
    Auction(RcAuctionKeys),
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

pub fn run() -> Result<()> {
    let Opts {
        thread_count,
        store_list,
        entries,
    } = Opts::parse();

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
                Job::GetStorefronts => get_storefronts::run(&client, handle),
                Job::GetBidderMetadata => bidder_metadata::get(&client, handle),
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

    let start_time = Local::now();

    let mut store_list = store_list;

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
            Entry::GetBidderMetadata => pool.push(Job::GetBidderMetadata),
        }
    }

    pool.join();

    let end_time = Local::now();

    let elapsed = {
        use std::fmt::Write;

        let duration = end_time - start_time;
        let mut out = String::new();

        let h = duration.num_hours();
        if h > 0 {
            write!(out, "{:02}:", h).unwrap();
        }

        write!(
            out,
            "{:02}:{:02}.{:03}",
            duration.num_minutes().rem_euclid(60),
            duration.num_seconds().rem_euclid(60),
            duration.num_milliseconds().rem_euclid(1000)
        )
        .unwrap();

        out
    };

    info!("Indexer run finished in {}", elapsed);

    Ok(())
}
