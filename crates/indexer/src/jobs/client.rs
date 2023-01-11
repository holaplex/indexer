use std::{path::PathBuf, sync::Arc};

use indexer_core::{clap, hash::DashMap};
use indexer_rabbitmq::{geyser, lapin, suffix::Suffix};
use indexer_selector::InstructionSelector;
use solana_client::rpc_client::RpcClient;

use crate::prelude::*;

/// Common arguments for job runner indexer usage
#[derive(Debug, clap::Args)]
#[group(skip)]
pub struct Args {
    /// Solana RPC endpoint
    #[arg(long, env)]
    solana_endpoint: String,

    /// Network hint for the Geyser AMQP queue.  Should match the network
    /// represented by [`solana_endpoint`](Self::solana_endpoint)
    #[arg(long, env)]
    network: geyser::Network,

    /// Path to a JSON file containing the Geyser selector configuration
    #[arg(long, env)]
    selector_config: PathBuf,
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
    geyser_chan: lapin::Channel,
    geyser_prod: DashMap<geyser::StartupType, geyser::Producer>,
    geyser_network: geyser::Network,
    suffix: Suffix,
    ins_sel: InstructionSelector,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails of the instruction selector configuration is invalid
    pub fn new_rc(geyser_chan: lapin::Channel, suffix: Suffix, args: Args) -> Result<Arc<Self>> {
        let Args {
            solana_endpoint,
            network,
            selector_config,
        } = args;

        let config_file = std::fs::File::open(&selector_config)
            .with_context(|| format!("Failed to open config file {selector_config:?}"))?;
        let config = serde_json::from_reader(config_file)
            .context("Failed to parse instruction selector config")?;

        Ok(Arc::new(Self {
            rpc: Rpc(RpcClient::new(solana_endpoint).into()),
            geyser_chan,
            geyser_prod: DashMap::default(),
            geyser_network: network,
            suffix,
            ins_sel: InstructionSelector::from_config(config)
                .context("Failed to construct instruction selector")?,
        }))
    }

    /// The configured instruction selector for block reindexing
    #[inline]
    #[must_use]
    pub fn ins_sel(&self) -> &InstructionSelector {
        &self.ins_sel
    }

    /// Dispatch an AMQP message to the Geyser indexer to upsert indexed
    /// on-chain data
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn dispatch_geyser(
        &self,
        startup: geyser::StartupType,
        msg: geyser::Message,
    ) -> Result<(), indexer_rabbitmq::Error> {
        use indexer_core::hash::dashmap::mapref::entry::Entry;

        if cfg!(debug_assertions) {
            error!("Refusing to submit Geyser message {msg:?} to {startup} queue on debug build");
            return Ok(());
        }

        let prod = match self.geyser_prod.entry(startup) {
            Entry::Occupied(o) => o.into_ref(),
            Entry::Vacant(v) => v.insert(
                geyser::Producer::from_channel(
                    self.geyser_chan.clone(),
                    geyser::QueueType::new(self.geyser_network, startup, &self.suffix)?,
                )
                .await?,
            ),
        };

        prod.write(msg).await
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
