use std::{borrow::Borrow, env, panic::AssertUnwindSafe, sync::Arc};

use indexer_core::{
    db::{Pool, PooledConnection},
    prelude::*,
};
use solana_client::{
    client_error::{ClientErrorKind, Result as ClientResult},
    rpc_client::RpcClient,
    rpc_config::RpcProgramAccountsConfig,
    rpc_request::RpcError,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling Solana JSONRPC client logic.
pub struct Client {
    db: AssertUnwindSafe<Pool>,
    rpc: AssertUnwindSafe<RpcClient>,
}

pub mod prelude {
    pub use solana_client::{
        rpc_config::RpcProgramAccountsConfig,
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    };
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if no `SOLANA_ENDPOINT` environment variable can be
    /// located.
    pub fn new_rc(db: Pool) -> Result<Arc<Self>> {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);

        Ok(Arc::new(Self {
            db: AssertUnwindSafe(db),
            rpc: AssertUnwindSafe(RpcClient::new(endpoint)),
        }))
    }

    /// Acquire a connection to the database
    ///
    /// # Errors
    /// This function fails if `r2d2` cannot acquire a database connection.
    pub fn db(&self) -> Result<PooledConnection> {
        self.db
            .0
            .get()
            .context("Failed to acquire database connection")
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
}
