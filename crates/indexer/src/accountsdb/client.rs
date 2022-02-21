use std::{borrow::Borrow, env, panic::AssertUnwindSafe, sync::Arc};

use indexer_core::prelude::*;
use indexer_rabbitmq::http_indexer;
use solana_client::{
    client_error::{ClientErrorKind, Result as ClientResult},
    rpc_client::RpcClient,
    rpc_config::RpcProgramAccountsConfig,
    rpc_request::RpcError,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::db::Pool;

pub mod prelude {
    pub use solana_client::{
        rpc_config::RpcProgramAccountsConfig,
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    };
}

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
    rpc: AssertUnwindSafe<RpcClient>,
    http: HttpProducers,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if no `SOLANA_ENDPOINT` environment variable can be
    /// located or if AMQP producers cannot be created for the given queue
    /// types.
    pub async fn new_rc(
        db: Pool,
        conn: &indexer_rabbitmq::lapin::Connection,
        meta_queue: http_indexer::QueueType<http_indexer::MetadataJson>,
        store_cfg_queue: http_indexer::QueueType<http_indexer::StoreConfig>,
    ) -> Result<Arc<Self>> {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);

        Ok(Arc::new(Self {
            db: AssertUnwindSafe(db),
            rpc: AssertUnwindSafe(RpcClient::new(endpoint)),
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
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Fetch a single Solana account.
    ///
    /// # Errors
    /// This function fails if the underlying JSONRPC call returns an error.
    // TODO: batch single requests
    pub fn get_account(&self, key: &Pubkey) -> ClientResult<Account> {
        self.rpc.0.get_account(key)
    }

    /// Fetch a single Solana account.
    ///
    /// # Errors
    /// This function fails if the underlying JSONRPC call returns an error.
    // TODO: batch single requests
    pub fn get_account_opt(&self, key: &Pubkey) -> ClientResult<Option<Account>> {
        match self.get_account(key) {
            Ok(a) => Ok(Some(a)),
            Err(e) if matches!(e.kind(), ClientErrorKind::RpcError(RpcError::ForUser(_))) => {
                Ok(None)
            },
            Err(e) => Err(e),
        }
    }

    /// Fetch multiple Solana accounts.
    ///
    /// # Errors
    /// This function fails if the underlying JSONRPC call returns an error.
    // TODO: merge batch requests when possible
    pub fn get_accounts(
        &self,
        keys: impl IntoIterator<Item = Pubkey>,
    ) -> ClientResult<Vec<Option<Account>>> {
        self.rpc
            .0
            .get_multiple_accounts(&*keys.into_iter().collect::<Vec<_>>())
    }

    /// Fetch multiple Solana accounts for a program given a config containing
    /// optional filters.
    ///
    /// # Errors
    /// This function fails if the underlying JSONRPC call returns an error.
    pub fn get_program_accounts(
        &self,
        program: impl Borrow<Pubkey>,
        config: RpcProgramAccountsConfig,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        self.rpc
            .0
            .get_program_accounts_with_config(program.borrow(), config)
    }

    /// Dispatch an AMQP message to the HTTP indexer to request off-chain
    /// metadata JSON
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn dispatch_metadata_json(
        &self,
        meta_address: Pubkey,
        uri: String,
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.http
            .metadata_json
            .write(http_indexer::MetadataJson { meta_address, uri })
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
