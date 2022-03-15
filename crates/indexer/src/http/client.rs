use std::sync::Arc;

use cid::Cid;
use indexer_core::assets::ArTxid;
use reqwest::Url;

use crate::{db::Pool, prelude::*};

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    ipfs_cdn: Url,
    arweave_cdn: Url,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if an invalid URL is given for `ipfs_cdn` or
    /// `arweave_cdn`.
    pub fn new_rc(db: Pool, ipfs_cdn: Url, arweave_cdn: Url) -> Result<Arc<Self>> {
        ensure!(!ipfs_cdn.cannot_be_a_base(), "Invalid IPFS CDN URL");
        ensure!(!arweave_cdn.cannot_be_a_base(), "Invalid Arweave CDN URL");

        Ok(Arc::new(Self {
            db,
            ipfs_cdn,
            arweave_cdn,
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
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
