use std::{sync::Arc, time::Duration};

use indexer_core::clap;
use indexer_rabbitmq::{http_indexer, search_indexer};
use solana_sdk::pubkey::Pubkey;

use crate::{db::Pool, prelude::*, reqwest, search_dispatch};

struct HttpProducers {
    metadata_json: http_indexer::Producer<http_indexer::MetadataJson>,
    store_config: http_indexer::Producer<http_indexer::StoreConfig>,
}

/// Common arguments for Geyser indexer usage
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Dialect API endpoint
    #[clap(long, env, requires("dialect-api-key"))]
    dialect_api_endpoint: Option<String>,

    /// Dialect API key
    #[clap(long, env, requires("dialect-api-endpoint"))]
    dialect_api_key: Option<String>,
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

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling networking logic
pub struct Client {
    db: Pool,
    http: reqwest::Client,
    http_prod: HttpProducers,
    search: search_dispatch::Client,
    dialect_api_endpoint: Option<String>,
    dialect_api_key: Option<String>,
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
        meta_queue: http_indexer::QueueType<http_indexer::MetadataJson>,
        store_cfg_queue: http_indexer::QueueType<http_indexer::StoreConfig>,
        search_queue: search_indexer::QueueType,
        Args {
            dialect_api_endpoint,
            dialect_api_key,
        }: Args,
    ) -> Result<Arc<Self>> {
        if dialect_api_endpoint.is_none() {
            warn!("Disabling Dialect integration");
        } else {
            debug!("Dialect integration enabled");
        }

        Ok(Arc::new(Self {
            db,
            http: reqwest::Client::new(Duration::from_millis(500))?,
            http_prod: HttpProducers {
                metadata_json: http_indexer::Producer::new(conn, meta_queue)
                    .await
                    .context("Couldn't create AMQP metadata JSON producer")?,
                store_config: http_indexer::Producer::new(conn, store_cfg_queue)
                    .await
                    .context("Couldn't create AMQP store config producer")?,
            },
            search: search_dispatch::Client::new(conn, search_queue).await?,
            dialect_api_endpoint,
            dialect_api_key,
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Get a reference to the search index dispatcher
    #[must_use]
    pub fn search(&self) -> &search_dispatch::Client {
        &self.search
    }

    /// Dispatch an AMQP message to the HTTP indexer to request off-chain
    /// metadata JSON
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn dispatch_metadata_json(
        &self,
        meta_address: Pubkey,
        first_verified_creator: Option<Pubkey>,
        uri: String,
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.http_prod
            .metadata_json
            .write(http_indexer::MetadataJson {
                meta_address,
                uri,
                first_verified_creator,
            })
            .await
    }

    /// Dispatch an AMQP message to the HTTP indexer to request off-chain store
    /// config data
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
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
            serde_json::to_string_pretty(&msg).unwrap_or_else(|e| format!("{:?}", e)),
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
