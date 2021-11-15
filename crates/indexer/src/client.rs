use std::{env, panic::AssertUnwindSafe, sync::Arc};

use indexer_core::prelude::*;
use solana_client::{
    client_error::{ClientErrorKind, Result as ClientResult},
    rpc_client::RpcClient,
    rpc_request::RpcError,
};
use solana_sdk::{account::Account, pubkey::Pubkey};

// RpcClient doesn't implement Debug for some reason
#[allow(missing_debug_implementations)]
/// Wrapper for handling Solana JSONRPC client logic.
pub struct Client {
    rpc: AssertUnwindSafe<RpcClient>,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if no `SOLANA_ENDPOINT` environment variable can be
    /// located.
    pub fn new_rc() -> Result<Arc<Self>> {
        let endpoint = env::var("SOLANA_ENDPOINT").context("Couldn't get Solana endpoint")?;
        info!("Connecting to endpoint: {:?}", endpoint);

        Ok(Arc::new(Self {
            rpc: AssertUnwindSafe(RpcClient::new(endpoint)),
        }))
    }

    /// Fetch a single Solana account.
    ///
    /// # Errors
    /// This function fails if the underlying JSONRPC call returns an error.
    // TODO: batch single requests
    pub fn get_account(&self, key: &Pubkey) -> ClientResult<Account> {
        (*self.rpc).get_account(key)
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
        (*self.rpc).get_multiple_accounts(&*keys.into_iter().collect::<Vec<_>>())
    }
}
