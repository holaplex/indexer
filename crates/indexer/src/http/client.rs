use std::{sync::Arc, time::Duration};

use indexer_core::{assets::AssetProxyArgs, clap};
use indexer_rabbitmq::search_indexer;
use serde_json::Value;

use crate::{db::Pool, prelude::*, reqwest};

/// Common arguments for internal HTTP indexer usage
#[derive(Debug, clap::Args)]
#[allow(missing_copy_implementations)]
pub struct Args {
    #[clap(flatten)]
    asset_proxy: AssetProxyArgs,

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
    search_prod: search_indexer::Producer,
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
        } = args;

        let timeout = Duration::from_secs_f64(timeout);

        Ok(Arc::new(Self {
            db,
            http: reqwest::Client::new(timeout)?,
            asset_proxy,
            search_prod: search_indexer::Producer::new(conn, search_queue)
                .await
                .context("Couldn't create AMQP search producer")?,
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

    /// Get a reference to the asset proxy arguments, used by
    /// [`proxy_url`](indexer_core::assets::proxy_url)
    #[inline]
    pub fn proxy_args(&self) -> &AssetProxyArgs {
        &self.asset_proxy
    }

    /// Dispatch an AMQP message to the Search indexer to index documents
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn dispatch_upsert_document(
        &self,
        id: String,
        index: String,
        body: Value,
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.search_prod
            .write(search_indexer::Message::Upsert {
                index,
                document: search_indexer::Document { id, body },
            })
            .await
    }
}
