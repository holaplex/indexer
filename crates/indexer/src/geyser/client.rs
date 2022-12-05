use std::{sync::Arc, time::Duration};

use indexer_core::clap;
use indexer_rabbitmq::{geyser, http_indexer, job_runner, search_indexer};

use crate::{db::Pool, prelude::*, reqwest, search_dispatch};

#[derive(Debug)]
struct HttpProducers {
    metadata_json: http_indexer::Producer<http_indexer::MetadataJson>,
    store_config: http_indexer::Producer<http_indexer::StoreConfig>,
}

#[derive(Debug)]
struct JobProducers {
    prod: job_runner::Producer,
    enable_block_reindex: bool,
}

/// Common arguments for Geyser indexer usage
#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Args {
    /// Dialect API endpoint
    #[arg(long, env, requires("dialect_api_key"))]
    dialect_api_endpoint: Option<String>,

    /// Dialect API key
    #[arg(long, env, requires("dialect_api_endpoint"))]
    dialect_api_key: Option<String>,

    /// Request reindexing of blocks when their status is marked as confirmed
    #[arg(long, env, default_value_t = false)]
    enable_block_reindex: bool,

    #[command(flatten)]
    search: search_dispatch::Args,
}

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
enum DialectEvent {
    #[serde(rename_all = "camelCase")]
    NftMakeOffer {
        bid_receipt_address: String,
        metadata_address: String,
    },
}

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    http: reqwest::Client,
    http_prod: HttpProducers,
    job_prod: JobProducers,
    search: search_dispatch::Client,
    startup: geyser::StartupType,
    dialect_api_endpoint: Option<String>,
    dialect_api_key: Option<String>,
}

/// Helper type for storing the necessary queue types for constructing a
/// [`Client`]
#[derive(Debug)]
pub struct Queues {
    /// HTTP queue for metadata JSON
    pub metadata_json: http_indexer::QueueType<http_indexer::MetadataJson>,
    /// HTTP queue for store config
    pub store_config: http_indexer::QueueType<http_indexer::StoreConfig>,
    /// Search indexer queue
    pub search: search_indexer::QueueType,
    /// Job runner queue
    pub jobs: job_runner::QueueType,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if AMQP producers cannot be created for the given queue
    /// types.
    pub async fn new_rc(
        db: Pool,
        conn: &indexer_rabbitmq::lapin::Connection,
        queues: Queues,
        startup: geyser::StartupType,
        Args {
            dialect_api_endpoint,
            dialect_api_key,
            enable_block_reindex,
            search,
        }: Args,
    ) -> Result<Arc<Self>> {
        if dialect_api_endpoint.is_none() {
            warn!("Disabling Dialect integration");
        } else {
            debug!("Dialect integration enabled");
        }

        let Queues {
            metadata_json: meta_q,
            store_config: store_q,
            search: search_q,
            jobs: job_q,
        } = queues;

        Ok(Arc::new(Self {
            db,
            http: reqwest::Client::new(Duration::from_millis(500))?,
            http_prod: HttpProducers {
                metadata_json: http_indexer::Producer::new(conn, meta_q)
                    .await
                    .context("Couldn't create AMQP metadata JSON producer")?,
                store_config: http_indexer::Producer::new(conn, store_q)
                    .await
                    .context("Couldn't create AMQP store config producer")?,
            },
            job_prod: JobProducers {
                prod: job_runner::Producer::new(conn, job_q)
                    .await
                    .context("Couldn't create job-runner producer")?,
                enable_block_reindex,
            },
            search: search_dispatch::Client::new(conn, search_q, search).await?,
            startup,
            dialect_api_endpoint,
            dialect_api_key,
        }))
    }

    /// Get a reference to the database
    #[inline]
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Get a reference to the search index dispatcher
    #[inline]
    #[must_use]
    pub fn search(&self) -> &search_dispatch::Client {
        &self.search
    }

    /// Dispatch an AMQP message to the HTTP indexer to request off-chain
    /// metadata JSON
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    #[inline]
    pub async fn dispatch_metadata_json(
        &self,
        meta_address: Pubkey,
        first_verified_creator: Option<Pubkey>,
        uri: String,
        slot_info: (u64, u64),
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.http_prod
            .metadata_json
            .write(http_indexer::MetadataJson {
                meta_address,
                uri,
                first_verified_creator,
                slot_info,
            })
            .await
    }

    /// Dispatch an AMQP message to the HTTP indexer to request off-chain store
    /// config data
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    #[inline]
    pub async fn dispatch_store_config(
        &self,
        config_address: Pubkey,
        uri: String,
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.http_prod
            .store_config
            .write(http_indexer::StoreConfig {
                config_address,
                uri,
            })
            .await
    }

    /// Dispatch an AMQP message to the job runner to reindex the block at the
    /// given slot
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    #[inline]
    pub async fn dispatch_block_reindex(&self, slot: u64) -> Result<(), indexer_rabbitmq::Error> {
        if !self.job_prod.enable_block_reindex {
            return Ok(());
        }

        let Self { startup, .. } = *self;

        self.job_prod
            .prod
            .write(job_runner::Message::ReindexSlot(job_runner::SlotReindex {
                slot,
                startup,
            }))
            .await
    }

    /// Dispatch a POST request to Dialect
    ///
    /// # Errors
    /// This function fails if the underlying POST request results in an error.
    pub async fn dispatch_dialect_offer_event(
        &self,
        bid_receipt_address: Pubkey,
        metadata_address: Pubkey,
    ) -> Result<()> {
        let (endpoint, key) =
            if let (Some(e), Some(k)) = (&self.dialect_api_endpoint, &self.dialect_api_key) {
                (e, k)
            } else {
                trace!(
                    "Dialect API args not present, skipping dispatch of bid {} on nft {}",
                    bid_receipt_address,
                    metadata_address
                );
                return Ok(());
            };

        let msg = DialectEvent::NftMakeOffer {
            bid_receipt_address: bid_receipt_address.to_string(),
            metadata_address: metadata_address.to_string(),
        };

        trace!(
            "Dispatching {} to Dialect at {:?}",
            serde_json::to_string_pretty(&msg).unwrap_or_else(|e| format!("{e:?}")),
            endpoint
        );

        let res = self
            .http
            .run(|h| {
                h.post(endpoint)
                    .basic_auth("holaplex", Some(key))
                    .json(&msg)
                    .send()
            })
            .await
            .context("Dialect dispatch call failed")?;

        if res.status().is_success() {
            trace!(
                "Dialect dispatch responded with {} ({:?})",
                res.status(),
                res.text().await
            );
        } else {
            warn!(
                "Dialect dispatch responded with non-success code {} ({:?})",
                res.status(),
                res.text().await
            );
        }

        Ok(())
    }
}
