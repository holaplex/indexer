//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

#[cfg(any(test, feature = "accountsdb"))]
pub mod accountsdb;
pub(crate) mod db;
#[cfg(any(test, feature = "http"))]
pub mod http;
pub(crate) mod util;

pub use runtime::{amqp_connect, run};

/// Common traits and re-exports
pub mod prelude {
    pub use indexer_core::prelude::*;
    pub use solana_sdk::{bs58, pubkey::Pubkey};
}

mod runtime {
    use std::{fmt::Debug, future::Future};

    use indexer_core::{
        clap,
        clap::{Args, Parser},
        db,
    };
    use indexer_rabbitmq::{consumer::Consumer, lapin, QueueType};
    use tokio_amqp::LapinTokioExt;

    use super::prelude::*;

    #[derive(Debug, Parser)]
    struct Opts<T: Debug + Args> {
        /// The number of threads to use.  Defaults to available core count.
        #[clap(short = 'j')]
        thread_count: Option<usize>,

        #[clap(flatten)]
        extra: T,
    }

    /// Entrypoint for `metaplex-indexer` binaries
    pub fn run<T: Debug + Args, F: Future<Output = Result<()>>>(
        f: impl FnOnce(T, db::Pool) -> F,
    ) -> ! {
        indexer_core::run(|| {
            let opts = Opts::parse();

            debug!("{:#?}", opts);

            let Opts {
                thread_count,
                extra,
            } = opts;

            let db =
                db::connect(db::ConnectMode::Write).context("Failed to connect to Postgres")?;

            let rt = {
                let mut b = tokio::runtime::Builder::new_multi_thread();

                if let Some(thread_count) = thread_count {
                    b.worker_threads(thread_count);
                }

                b.enable_all()
                    .build()
                    .context("Failed to initialize async runtime")?
            };

            rt.block_on(f(extra, db))
        })
    }

    /// Create a new AMQP connection from the given URL
    ///
    /// # Errors
    /// This function fails if a connection cannot be established
    pub async fn amqp_connect(addr: impl AsRef<str>) -> Result<lapin::Connection> {
        lapin::Connection::connect(
            addr.as_ref(),
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await
        .context("Failed to connect to the AMQP server")
    }
}
