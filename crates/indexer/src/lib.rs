//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub mod db;
#[cfg(any(test, feature = "geyser"))]
pub mod geyser;
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
    use std::{fmt::Debug, future::Future};

    use futures_util::{FutureExt, StreamExt};
    use indexer_core::{
        clap,
        clap::{Args, Parser},
        db,
    };
    use indexer_rabbitmq::{
        consumer::Consumer,
        lapin,
        lapin::options::{BasicAckOptions, BasicRejectOptions},
        QueueType,
    };
    use tokio::sync::{broadcast, broadcast::error::RecvError};

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

    /// Entrypoint for `holaplex-indexer` binaries
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
        Q: QueueType + Send + Sync + 'static,
        F: Send + Future<Output = Result<()>> + 'static,
    >(
        params: &Params,
        conn: indexer_rabbitmq::lapin::Connection,
        consumer: Consumer<Q>,
        queue_type: Q,
        process: impl Fn(Q::Message) -> F + Send + Sync + Clone + 'static,
    ) -> Result<()>
    where
        Q::Message: Debug + Send + for<'a> serde::Deserialize<'a>,
    {
        async fn consume_one<Q: QueueType, F: Future<Output = Result<()>>>(
            worker_id: usize,
            mut consumer: Consumer<Q>,
            process: impl Fn(Q::Message) -> F,
            mut stop_rx: broadcast::Receiver<()>,
        ) -> Result<()>
        where
            Q::Message: Debug + for<'de> serde::Deserialize<'de>,
        {
            // Ideally T would be ! but ! is unstable.
            fn handle_stop<T>(r: Result<(), RecvError>) -> Result<Option<T>> {
                match r {
                    Ok(()) | Err(RecvError::Closed) => Ok(None),
                    Err(e) => Err(e).context("Error receiving stop signal"),
                }
            }

            loop {
                let val = tokio::select! {
                    r = consumer.read() => r.context("Failed to read AMQP message")?,
                    r = stop_rx.recv() => handle_stop(r)?,
                };

                let (msg, acker) = if let Some(del) = val {
                    del
                } else {
                    break;
                };

                trace!("Worker {}: {:?}", worker_id, msg);

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
            }

            Ok(())
        }

        let Params { concurrency } = *params;

        let dl_task = tokio::spawn(indexer_rabbitmq::dl_consumer::run(
            conn,
            queue_type,
            tokio::time::sleep,
        ));

        let (stop_tx, _stop_rx) = broadcast::channel(1);

        let mut q_tasks = (0..concurrency)
            .map(|i| {
                tokio::spawn(consume_one(
                    i,
                    consumer.clone(),
                    process.clone(),
                    stop_tx.subscribe(),
                ))
                .map(|r| match r {
                    Ok(Ok(())) => warn!("AMQP server hung up!"),
                    Ok(Err(e)) => error!("Fatal error in worker: {:?}", e),
                    Err(e) => error!("Worker terminated unexpectedly: {:?}", e),
                })
            })
            .collect::<futures_util::stream::FuturesUnordered<_>>();

        q_tasks.next().await; // Everything past this point is graceful failure

        stop_tx.send(()).unwrap();
        dl_task.abort();

        if !q_tasks.is_empty() {
            info!("Waiting for additional jobs to finish...");
        }

        while let Some(()) = q_tasks.next().await {}

        std::mem::drop(stop_tx);

        dl_task
            .await
            .map_err(|e| error!("DLX consumer cleanup failed: {:?}", e))
            .unwrap_or(());

        Ok(())
    }
}
