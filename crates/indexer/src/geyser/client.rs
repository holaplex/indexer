use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::http_indexer;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::{db::Pool, reqwest};

struct HttpProducers {
    metadata_json: http_indexer::Producer<http_indexer::MetadataJson>,
    store_config: http_indexer::Producer<http_indexer::StoreConfig>,
}

impl std::panic::UnwindSafe for HttpProducers {}
impl std::panic::RefUnwindSafe for HttpProducers {}

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

#[derive(Serialize, Deserialize)]
enum DialectEventType {
    NftMakeOffer,
}

#[derive(Serialize, Deserialize)]
struct DialectOfferEventData {
    bid_receipt_address: String,
    metadata_address: String,
}

#[derive(Serialize, Deserialize)]
enum DialectEventData {
    DialectOfferEventData(DialectOfferEventData),
}
#[derive(Serialize, Deserialize)]
struct DialectEvent {
    event_type: DialectEventType,
    data: DialectEventData,
}

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling networking logic
pub struct Client {
    db: AssertUnwindSafe<Pool>,
    http: reqwest::Client,
    http_prod: HttpProducers,
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
        Args {
            dialect_api_endpoint,
            dialect_api_key,
        }: Args,
    ) -> Result<Arc<Self>> {
        if dialect_api_endpoint.is_none() {
            warn!("Disabling Dialect integration");
        }

        Ok(Arc::new(Self {
            db: AssertUnwindSafe(db),
            http: reqwest::Client::new(Duration::from_millis(500))?,
            http_prod: HttpProducers {
                metadata_json: http_indexer::Producer::new(conn, meta_queue)
                    .await
                    .context("Couldn't create AMQP metadata JSON producer")?,
                store_config: http_indexer::Producer::new(conn, store_cfg_queue)
                    .await
                    .context("Couldn't create AMQP store config producer")?,
            },
            dialect_api_endpoint,
            dialect_api_key,
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
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

        let msg = DialectEvent {
            event_type: DialectEventType::NftMakeOffer,
            data: DialectEventData::DialectOfferEventData(DialectOfferEventData {
                bid_receipt_address: bid_receipt_address.to_string(),
                metadata_address: metadata_address.to_string(),
            }),
        };

        self.http
            .run(|h| {
                h.post(endpoint)
                    .basic_auth("holaplex", Some(key))
                    .json(&msg)
                    .send()
            })
            .await?;

        Ok(())
    }
}
