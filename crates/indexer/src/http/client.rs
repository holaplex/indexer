use std::{sync::Arc, time::Duration};

use cid::Cid;
use indexer_core::{assets::ArTxid, clap};
use reqwest::Url;

use super::http_client_cache::HttpClientCache;
use crate::{db::Pool, prelude::*, Params};

/// Common arguments for internal HTTP indexer usage
#[derive(Debug, clap::Parser)]
#[allow(missing_copy_implementations)]
pub struct Args {
    /// A valid base URL to use when fetching IPFS links
    #[clap(long, env)]
    pub ipfs_cdn: String,

    /// A valid base URL to use when fetching Arweave links
    #[clap(long, env)]
    pub arweave_cdn: String,

    /// HTTP request timeout, in seconds
    #[clap(long, env = "HTTP_INDEXER_TIMEOUT")]
    pub timeout: f64,
}

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    http_clients: HttpClientCache,
    ipfs_cdn: Url,
    arweave_cdn: Url,
    timeout: Duration,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if an invalid URL is given for `ipfs_cdn` or
    /// `arweave_cdn`.
    pub fn new_rc(db: Pool, args: Args, params: &Params) -> Result<Arc<Self>> {
        let Args {
            ipfs_cdn,
            arweave_cdn,
            timeout,
        } = args;

        let ipfs_cdn: Url = ipfs_cdn.parse().context("Failed to parse IPFS CDN URL")?;
        let arweave_cdn: Url = arweave_cdn
            .parse()
            .context("Failed to parse Arweave CDN URL")?;

        ensure!(!ipfs_cdn.cannot_be_a_base(), "Invalid IPFS CDN URL");
        ensure!(!arweave_cdn.cannot_be_a_base(), "Invalid Arweave CDN URL");

        Ok(Arc::new(Self {
            db,
            http_clients: HttpClientCache::new(params.concurrency),
            ipfs_cdn,
            arweave_cdn,
            timeout: Duration::from_secs_f64(timeout),
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Timeout hint for indexer HTTP requests
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Acquire an HTTP client, using a URL hint to select a cache
    ///
    /// # Errors
    /// This function returns an error if an HTTP client could not be acquired
    /// or constructed.
    #[inline]
    pub fn http(&self, url: &Url) -> Result<super::http_client_cache::CachedClient> {
        self.http_clients.acquire(url)
    }

    /// Construct an IPFS link from an IPFS CID
    ///
    /// # Errors
    /// This function fails if the CID provided is not URL safe.
    pub fn ipfs_link(&self, cid: &Cid) -> Result<Url> {
        self.ipfs_cdn.join(&cid.to_string()).map_err(Into::into)
    }

    /// Construct an Arweave link from a valid Arweave transaction ID
    ///
    /// # Errors
    /// This function fails if the transaction ID provided is not URL safe
    pub fn arweave_link(&self, txid: &ArTxid) -> Result<Url> {
        self.arweave_cdn
            .join(&base64::encode_config(&txid.0, base64::URL_SAFE_NO_PAD))
            .map_err(Into::into)
    }
}
