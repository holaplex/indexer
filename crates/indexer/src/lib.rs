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
pub mod db;
#[cfg(any(test, feature = "http"))]
pub mod http;
#[cfg(any(test, feature = "http"))]
pub mod legacy_storefronts;
pub(crate) mod util;

pub use runtime::*;

/// Common traits and re-exports
pub mod prelude {
    pub use indexer_core::prelude::*;
    pub use solana_sdk::{bs58, pubkey::Pubkey};
}

mod runtime {
    use std::{fmt::Debug, future::Future, sync::Arc};

    use futures_util::{FutureExt, StreamExt};
    use indexer_core::{
        clap,
        clap::{Args, Parser},
        db,
    };
    use indexer_rabbitmq::lapin;
    use tokio::sync::Semaphore;

    use super::{db::Pool, prelude::*};

    #[derive(Debug, Parser)]
    struct Opts<T: Debug + Args> {
        /// The number of threads to use.  Defaults to available core count.
        #[clap(short = 'j')]
        thread_count: Option<usize>,

        #[clap(flatten)]
        extra: T,
    }

    /// Common parameters for all indexers
    #[allow(missing_copy_implementations)]
    #[derive(Debug)]
    pub struct Params {
        concurrency: usize,
    }

    /// Entrypoint for `metaplex-indexer` binaries
    pub fn run<T: Debug + Args, F: Future<Output = Result<()>>>(
        f: impl FnOnce(T, Params, Pool) -> F,
    ) -> ! {
        indexer_core::run(|| {
            let opts = Opts::parse();

            debug!("{:#?}", opts);

            let Opts {
                thread_count,
                extra,
            } = opts;

            let db = Pool::new(
                db::connect(db::ConnectMode::Write).context("Failed to connect to Postgres")?,
            );

            let rt = {
                let mut b = tokio::runtime::Builder::new_multi_thread();

                if let Some(thread_count) = thread_count {
                    b.worker_threads(thread_count);
                }

                b.enable_all()
                    .build()
                    .context("Failed to initialize async runtime")?
            };

            let concurrency = thread_count.unwrap_or_else(num_cpus::get);

            rt.block_on(f(extra, Params { concurrency }, db))
        })
    }

    /// Create a new AMQP connection from the given URL
    ///
    /// # Errors
    /// This function fails if a connection cannot be established
    pub async fn amqp_connect(addr: impl AsRef<str>) -> Result<lapin::Connection> {
        lapin::Connection::connect(
            addr.as_ref(),
            lapin::ConnectionProperties::default()
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await
        .context("Failed to connect to the AMQP server")
    }

    /// Consume messages from an AMQP consumer until the connection closes
    ///
    /// # Errors
    /// This function fails if a message cannot be received, but _does not_ fail
    /// if a received message fails to process.
    ///
    /// # Panics
    /// This function will panic if the internal scheduler enters a deadlock
    /// state.
    pub async fn amqp_consume<
        Q: indexer_rabbitmq::QueueType + Send + Sync + 'static,
        F: Send + Future<Output = Result<()>> + 'static,
    >(
        params: &Params,
        conn: indexer_rabbitmq::lapin::Connection,
        mut consumer: indexer_rabbitmq::consumer::Consumer<Q>,
        queue_type: Q,
        process: impl Fn(Q::Message) -> F,
    ) -> Result<()>
    where
        Q::Message: Debug + for<'a> serde::Deserialize<'a>,
    {
        type JobResult = (
            Result<Result<()>, tokio::task::JoinError>,
            indexer_rabbitmq::lapin::acker::Acker,
        );

        async fn finish_job((res, acker): JobResult) -> Result<()> {
            use indexer_rabbitmq::lapin::options::{BasicAckOptions, BasicRejectOptions};

            match res {
                Ok(Ok(())) => acker
                    .ack(BasicAckOptions::default())
                    .await
                    .context("Failed to send ACK for delivery"),
                Ok(Err(e)) => {
                    warn!("Failed to process message: {:?}", e);

                    acker
                        .reject(BasicRejectOptions { requeue: false })
                        .await
                        .context("Failed to send NAK for delivery")
                },
                Err(e) => {
                    warn!("Could not gracefully join worker task: {:?}", e);

                    Ok(())
                },
            }
        }

        let Params { concurrency } = *params;
        let mut futures = futures_util::stream::FuturesUnordered::new();
        let sem = Arc::new(Semaphore::new(concurrency));

        let dl_task = tokio::spawn(indexer_rabbitmq::dl_consumer::run(
            conn,
            queue_type,
            tokio::time::sleep,
        ));

        loop {
            enum Message {
                Permit(Result<tokio::sync::OwnedSemaphorePermit, tokio::sync::AcquireError>),
                JobDone(JobResult),
            }

            let msg = tokio::select! {
                p = sem.clone().acquire_owned() => Message::Permit(p),
                Some(r) = futures.next() => Message::JobDone(r),
            };

            match msg {
                Message::Permit(p) => {
                    if let Some((msg, acker)) = consumer
                        .read()
                        .await
                        .context("Failed to read AMQP message")?
                    {
                        trace!("{:?}", msg);

                        let process = process(msg);

                        futures.push(
                            tokio::task::spawn(async move {
                                let res = process.await;

                                std::mem::drop(p); // Hang on to the permit until here

                                res
                            })
                            .map(|r| (r, acker)),
                        );
                    } else {
                        warn!("AMQP server hung up!");
                        break;
                    }
                },
                Message::JobDone(r) => finish_job(r).await?,
            }
        }

        dl_task.abort();

        if !futures.is_empty() {
            info!("Waiting for additional jobs to finish...");
        }

        while let Some(r) = futures.next().await {
            finish_job(r)
                .await
                .map_err(|e| error!("Job cleanup failed: {:?}", e))
                .unwrap_or(());
        }

        dl_task
            .await
            .map_err(|e| error!("DLX consumer cleanup failed: {:?}", e))
            .unwrap_or(());

        Ok(())
    }
}
