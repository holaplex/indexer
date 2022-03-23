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

    use futures_util::StreamExt;
    use indexer_core::{
        clap,
        clap::{Args, Parser},
        db,
    };
    use indexer_rabbitmq::{
        lapin,
        lapin::options::{BasicAckOptions, BasicRejectOptions},
    };
    use tokio::{sync::Semaphore, task::JoinError};

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
        consumer: indexer_rabbitmq::consumer::Consumer<Q>,
        queue_type: Q,
        process: impl Fn(Q::Message) -> F + Send + Sync + 'static,
    ) -> Result<()>
    where
        Q::Message: Debug + Send + for<'a> serde::Deserialize<'a>,
    {
        enum JobResult {
            Continue,
            Hangup,
        }

        let Params { concurrency } = *params;
        let mut futures = futures_util::stream::FuturesUnordered::new();
        let sem = Arc::new(Semaphore::new(concurrency));
        let process = Arc::new(process);

        let dl_task = tokio::spawn(indexer_rabbitmq::dl_consumer::run(
            conn,
            queue_type,
            tokio::time::sleep,
        ));

        loop {
            enum Message {
                Permit(Result<tokio::sync::OwnedSemaphorePermit, tokio::sync::AcquireError>),
                JobDone(Result<Result<JobResult>, JoinError>),
                Heartbeat,
            }

            let heartbeat_interval = std::time::Duration::from_secs(5);

            let msg = tokio::select! {
                p = sem.clone().acquire_owned() => Message::Permit(p),
                Some(r) = futures.next() => Message::JobDone(r),
                _ = tokio::time::sleep(heartbeat_interval) => Message::Heartbeat,
            };

            match msg {
                Message::Permit(p) => {
                    let mut consumer = consumer.clone();
                    let process = Arc::clone(&process);

                    futures.push(tokio::task::spawn(async move {
                        if let Some((msg, acker)) = consumer
                            .read()
                            .await
                            .context("Failed to read AMQP message")?
                        {
                            trace!("{:?}", msg);

                            match process(msg).await {
                                Ok(()) => acker
                                    .ack(BasicAckOptions::default())
                                    .await
                                    .context("Failed to send ACK for delivery")?,
                                Err(e) => {
                                    warn!("Failed to process message: {:?}", e);

                                    acker
                                        .reject(BasicRejectOptions { requeue: false })
                                        .await
                                        .context("Failed to send NAK for delivery")?;
                                },
                            }

                            std::mem::drop(p); // Hang on to the permit until here

                            Ok(JobResult::Continue)
                        } else {
                            warn!("AMQP server hung up!");

                            Ok(JobResult::Hangup)
                        }
                    }));
                },
                Message::JobDone(Ok(Ok(JobResult::Continue))) => (),
                Message::JobDone(Ok(Ok(JobResult::Hangup))) => break,
                Message::JobDone(Ok(Err(e))) => {
                    error!("Fatal error encountered from worker: {:?}", e);
                    break;
                },
                Message::JobDone(Err(e)) => error!("Failed to join worker: {:?}", e),
                Message::Heartbeat => {
                    debug!(
                        "Heartbeat received (deadlock?) sem = {}, futures = {}",
                        sem.available_permits(),
                        futures.len(),
                    );
                },
            }
        }

        sem.close();
        dl_task.abort();

        if !futures.is_empty() {
            info!("Waiting for additional jobs to finish...");
        }

        while let Some(r) = futures.next().await {
            r.map_err(Into::into)
                .and_then(|r| r)
                .map_err(|e| error!("Job cleanup failed: {:?}", e))
                .ok();
        }

        dl_task
            .await
            .map_err(|e| error!("DLX consumer cleanup failed: {:?}", e))
            .unwrap_or(());

        Ok(())
    }
}
