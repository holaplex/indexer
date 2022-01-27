//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

mod bits;
mod client;
mod entry;
pub(crate) mod util;

pub(crate) use client::Client;
pub use entry::process_message;
pub(crate) use entry::{AuctionCacheKeys, EditionKeys, ListingMetadata, RcAuctionKeys};
pub use runtime::{create_consumer, run};

pub mod prelude {
    pub use indexer_core::prelude::*;
    pub use solana_sdk::{bs58, pubkey::Pubkey};
}

mod runtime {
    use std::{fmt::Debug, future::Future, sync::Arc};

    use indexer_core::{
        clap,
        clap::{Args, Parser},
        db,
    };
    use indexer_rabbitmq::{consumer::Consumer, lapin, QueueType};
    use tokio_amqp::LapinTokioExt;

    use super::prelude::*;
    use crate::Client;

    #[derive(Debug, Parser)]
    struct Opts<T: Debug + Args> {
        /// A valid base URL to use when fetching IPFS links
        #[clap(long, env)]
        ipfs_cdn: Option<String>,

        /// A valid base URL to use when fetching Arweave links
        #[clap(long, env)]
        arweave_cdn: Option<String>,

        /// The number of threads to use.  Defaults to available core count.
        #[clap(short = 'j')]
        thread_count: Option<usize>,

        #[clap(flatten)]
        extra: T,
    }

    /// Entrypoint for `metaplex-indexer` binaries
    pub fn run<T: Debug + Args, F: Future<Output = Result<()>>>(
        f: impl FnOnce(T, Arc<Client>) -> F,
    ) -> ! {
        indexer_core::run(|| {
            let opts = Opts::parse();

            debug!("{:#?}", opts);

            let Opts {
                arweave_cdn,
                ipfs_cdn,
                thread_count,
                extra,
            } = opts;

            let db =
                db::connect(db::ConnectMode::Write).context("Failed to connect to Postgres")?;

            let client = Client::new_rc(
                db,
                ipfs_cdn
                    .ok_or_else(|| anyhow!("Missing IPFS CDN"))?
                    .parse()
                    .context("Failed to parse IPFS CDN URL")?,
                arweave_cdn
                    .ok_or_else(|| anyhow!("Missing Arweave CDN"))?
                    .parse()
                    .context("Failed to parse Arweave CDN URL")?,
            )
            .context("Failed to construct Client")?;

            let rt = {
                let mut b = tokio::runtime::Builder::new_multi_thread();

                if let Some(thread_count) = thread_count {
                    b.worker_threads(thread_count);
                }

                b.enable_io()
                    .build()
                    .context("Failed to initialize async runtime")?
            };

            rt.block_on(f(extra, client))
        })
    }

    pub async fn create_consumer<T: for<'a> serde::Deserialize<'a>, Q: QueueType<T>>(
        addr: impl AsRef<str>,
        queue_type: Q,
    ) -> Result<Consumer<T, Q>> {
        let conn = lapin::Connection::connect(
            addr.as_ref(),
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await
        .context("Failed to connect to the AMQP server")?;

        Consumer::new(&conn, queue_type)
            .await
            .context("Failed to create a queue consumer")
    }
}
