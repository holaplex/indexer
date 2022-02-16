use std::{borrow::Borrow, env, panic::AssertUnwindSafe, sync::Arc};

use indexer_core::{
    db::{Pool, PooledConnection},
    prelude::*,
};
use indexer_rabbitmq::http_indexer;
use solana_client::{
    client_error::{ClientErrorKind, Result as ClientResult},
    rpc_client::RpcClient,
    rpc_config::RpcProgramAccountsConfig,
    rpc_request::RpcError,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

pub mod prelude {
    pub use solana_client::{
        rpc_config::RpcProgramAccountsConfig,
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    };
}

/// An Arweave transaction ID
#[derive(Debug, Clone, Copy)]
pub struct ArTxid(pub [u8; 32]);

struct HttpProducers {
    metadata_json: http_indexer::Producer<http_indexer::MetadataJson>,
    store_config: http_indexer::Producer<http_indexer::StoreConfig>,
}

impl std::panic::UnwindSafe for HttpProducers {}
impl std::panic::RefUnwindSafe for HttpProducers {}

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling Solana JSONRPC client logic.
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
    /// located.
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

    /// Spawn a blocking thread to perform operations on the database.
    ///
    /// # Errors
    /// This function fails if `r2d2` cannot acquire a database connection or
    /// the provided callback returns an error.
    pub async fn db<T: 'static + Send, E: 'static + Into<indexer_core::error::Error>>(
        &self,
        f: impl FnOnce(&PooledConnection) -> Result<T, E> + Send + 'static,
    ) -> Result<T> {
        let db = self
            .db
            .0
            .get()
            .context("Failed to acquire database connection");

        tokio::task::spawn_blocking(|| f(&db?).map_err(Into::into))
            .await
            .context("Blocking task failed")?
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

    pub async fn dispatch_store_config(
        &self,
        store_address: Pubkey,
        uri: String,
    ) -> Result<(), indexer_rabbitmq::Error> {
        self.http
            .store_config
            .write(http_indexer::StoreConfig { store_address, uri })
            .await
    }

    // /// Construct an IPFS link from an IPFS CID
    // ///
    // /// # Errors
    // /// This function fails if the CID provided is not URL safe.
    // pub fn ipfs_link(&self, cid: &Cid) -> Result<Url> {
    //     self.ipfs_cdn.join(&cid.to_string()).map_err(Into::into)
    // }

    // /// Construct an Arweave link from a valid Arweave transaction ID
    // ///
    // /// # Errors
    // /// This function fails if the transaction ID provided is not URL safe
    // pub fn arweave_link(&self, txid: &ArTxid) -> Result<Url> {
    //     self.arweave_cdn
    //         .join(&base64::encode_config(&txid.0, base64::URL_SAFE_NO_PAD))
    //         .map_err(Into::into)
    // }
}
