use std::sync::Arc;

use indexer_core::clap;
use solana_client::rpc_client::RpcClient;

use crate::prelude::*;

/// Common arguments for job runner indexer usage
#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Args {
    /// Solana RPC endpoint
    #[arg(long, env)]
    solana_endpoint: String,
}

// rpc_client::RpcClient doesn't implement Debug for some reason
#[derive(Clone)]
#[repr(transparent)]
pub struct Rpc(Arc<RpcClient>);

impl std::fmt::Debug for Rpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcClient").finish_non_exhaustive()
    }
}

impl std::ops::Deref for Rpc {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    rpc: Rpc,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    #[must_use]
    pub fn new_rc(args: Args) -> Arc<Self> {
        let Args { solana_endpoint } = args;

        Arc::new(Self {
            rpc: Rpc(RpcClient::new(solana_endpoint).into()),
        })
    }

    /// Spawn a blocking thread to perform RPC operations.
    ///
    /// # Errors
    /// This function fails if the provided callback returns an error or the
    /// blocking thread cannot be scheduled correctly.
    pub async fn run_rpc<T: Send + 'static, E: Into<indexer_core::error::Error> + 'static>(
        &self,
        f: impl FnOnce(&Rpc) -> Result<T, E> + Send + 'static,
    ) -> Result<T> {
        let rpc = self.rpc.clone();

        tokio::task::spawn_blocking(move || f(&rpc).map_err(Into::into))
            .await
            .context("Blocking task failed")?
    }
}
