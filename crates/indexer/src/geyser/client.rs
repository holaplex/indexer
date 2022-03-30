use std::{panic::AssertUnwindSafe, sync::Arc};

use indexer_core::prelude::*;
use indexer_rabbitmq::http_indexer;
use solana_sdk::pubkey::Pubkey;

use crate::db::Pool;

struct HttpProducers {
    metadata_json: http_indexer::Producer<http_indexer::MetadataJson>,
    store_config: http_indexer::Producer<http_indexer::StoreConfig>,
}

impl std::panic::UnwindSafe for HttpProducers {}
impl std::panic::RefUnwindSafe for HttpProducers {}

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling networking logic
pub struct Client {
    db: AssertUnwindSafe<Pool>,
    http: HttpProducers,
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
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            db: AssertUnwindSafe(db),
            http: HttpProducers {
                metadata_json: http_indexer::Producer::new(conn, meta_queue)
                    .await
                    .context("Couldn't create AMQP metadata JSON producer")?,
                store_config: http_indexer::Producer::new(conn, store_cfg_queue)
                    .await
                    .context("Couldn't create AMQP store config producer")?,
            },
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
        self.http
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
        self.http
            .store_config
            .write(http_indexer::StoreConfig {
                config_address,
                uri,
            })
            .await
    }
}
