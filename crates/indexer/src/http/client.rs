use std::{sync::Arc, time::Duration};

use indexer_core::{assets::AssetProxyArgs, clap};
use indexer_rabbitmq::search_indexer;

use crate::{db::Pool, prelude::*, reqwest, search_dispatch};

/// Common arguments for internal HTTP indexer usage
#[derive(Debug, clap::Args)]
#[allow(missing_copy_implementations)]
pub struct Args {
    #[clap(flatten)]
    asset_proxy: AssetProxyArgs,

    #[clap(flatten)]
    search: search_dispatch::Args,

    /// HTTP request timeout, in seconds
    #[clap(long, env = "HTTP_INDEXER_TIMEOUT")]
    timeout: f64,
}

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    http: reqwest::Client,
    asset_proxy: AssetProxyArgs,
    search: search_dispatch::Client,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if an invalid URL is given for `ipfs_cdn` or
    /// `arweave_cdn`.
    pub async fn new_rc(
        db: Pool,
        conn: &indexer_rabbitmq::lapin::Connection,
        args: Args,
        search_queue: search_indexer::QueueType,
    ) -> Result<Arc<Self>> {
        let Args {
            asset_proxy,
            timeout,
            search,
        } = args;

        let timeout = Duration::from_secs_f64(timeout);

        Ok(Arc::new(Self {
            db,
            http: reqwest::Client::new(timeout)?,
            asset_proxy,
            search: search_dispatch::Client::new(conn, search_queue, search).await?,
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Get a reference to the HTTP client
    #[inline]
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Get a reference to the search index dispatcher
    pub fn search(&self) -> &search_dispatch::Client {
        &self.search
    }

    /// Get a reference to the asset proxy arguments, used by
    /// [`proxy_url`](indexer_core::assets::proxy_url)
    #[inline]
    pub fn proxy_args(&self) -> &AssetProxyArgs {
        &self.asset_proxy
    }
}
