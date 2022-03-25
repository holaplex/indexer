use std::{sync::Arc, time::Duration};

use cid::Cid;
use indexer_core::{assets::ArTxid, clap};
use reqwest::Url;

use crate::{db::Pool, prelude::*};

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
    http: reqwest::Client,
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
    pub fn new_rc(db: Pool, args: Args) -> Result<Arc<Self>> {
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
            http: reqwest::Client::new(),
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

    /// Acquire an HTTP client
    #[inline]
    #[must_use]
    pub fn http(&self) -> reqwest::Client {
        self.http.clone()
    }

    /// Construct an IPFS link from an IPFS CID
    ///
    /// # Errors
    /// This function fails if the CID provided is not URL safe.
    pub fn ipfs_link(&self, cid: &Cid, path: &str) -> Result<Url> {
        let mut ret = self.ipfs_cdn.clone();

        {
            let mut parts = ret
                .path_segments_mut()
                .map_err(|_| anyhow!("Invalid IPFS CDN URL"))?;

            parts.push(&cid.to_string());

            if !path.is_empty() {
                parts.extend(path.split('/'));
            }
        }

        Ok(ret)
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
