//! Binary for running the write half of the indexer.

#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub mod db;
#[cfg(feature = "geyser")]
pub mod geyser;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "job-runner")]
pub mod jobs;
#[cfg(feature = "reqwest")]
pub(crate) mod reqwest;
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "search-dispatch")]
/// Search dispatch module for creating client and dispatching AMQP messages to the search indexer
pub mod search_dispatch;
pub(crate) mod util;

pub use runtime::*;

/// Common traits and re-exports
pub mod prelude {
    pub use bs58;
    pub use indexer_core::prelude::*;
    pub use solana_program::pubkey::Pubkey;

    pub use crate::{MessageError, MessageResult};
}

mod runtime {
    use std::{
        fmt::{Debug, Display},
        future::Future,
    };

    use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};
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
        #[arg(short = 'j', env)]
        thread_count: Option<usize>,

        /// Pass this flag to enable automatically migrating the database upon
        /// connecting.
        #[arg(long, short, env)]
        migrate_db: bool,

        #[command(flatten)]
        db: db::ConnectArgs,

        #[command(flatten)]
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
                db,
                migrate_db: migrate,
                extra,
            } = opts;

            let db = Pool::new(
                db::connect(db, db::ConnectMode::Write { migrate })
                    .context("Failed to connect to Postgres")?,
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

            let concurrency = thread_count.unwrap_or_else(indexer_core::num_cpus::get);

            rt.block_on(f(extra, Params { concurrency }, db))
        })
    }

    /// Create a new AMQP connection from the given URL
    ///
    /// # Errors
    /// This function fails if a connection cannot be established
    pub async fn amqp_connect(
        addr: impl AsRef<str>,
        name: &'static str,
    ) -> Result<lapin::Connection> {
        lapin::Connection::connect(
            addr.as_ref(),
            lapin::ConnectionProperties::default()
                .with_connection_name(
                    format!(
                        "{}@{}",
                        name,
                        hostname::get()
                            .context("Failed to get system hostname")?
                            .into_string()
                            .map_err(|_| anyhow!("Failed to parse system hostname"))?,
                    )
                    .into(),
                )
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await
        .context("Failed to connect to the AMQP server")
    }

    enum StopType {
        Hangup,
        Stopped,
    }

    /// An error from a message processor, including a message identifier
    #[derive(Debug, thiserror::Error)]
    #[error("Failed to process {1}: {0:?}")]
    pub struct MessageError<D>(#[source] Error, D);

    impl<D: Display> MessageError<D> {
        /// Construct a new message error
        #[inline]
        #[must_use]
        pub fn new(err: Error, id: D) -> Self {
            Self(err, id)
        }
    }

    /// Convenience alias for the result of a message processor function
    pub type MessageResult<D> = Result<(), MessageError<D>>;

    async fn consume_one<Q: QueueType, F: Future<Output = MessageResult<D>>, D: Display>(
        worker_id: usize,
        mut consumer: Consumer<Q>,
        process: impl Fn(Q::Message) -> F,
        mut stop_rx: broadcast::Receiver<()>,
    ) -> Result<StopType>
    where
        Q::Message: Debug + for<'de> serde::Deserialize<'de>,
    {
        enum Delivery<T> {
            Message(Result<Option<(T, lapin::acker::Acker)>, indexer_rabbitmq::Error>),
            Stop,
        }

        // Ideally T would be ! but ! is unstable.
        fn handle_stop<T>(r: Result<(), RecvError>) -> Result<Delivery<T>> {
            match r {
                Ok(()) | Err(RecvError::Closed) => Ok(Delivery::Stop),
                Err(e) => Err(e).context("Error receiving stop signal"),
            }
        }

        loop {
            let del = tokio::select! {
                r = consumer.read() => Delivery::Message(r),
                r = stop_rx.recv() => handle_stop(r)?,
            };

            let (msg, acker) = match del {
                Delivery::Message(Ok(Some(d))) => d,
                Delivery::Message(Ok(None)) => break Ok(StopType::Hangup),
                Delivery::Message(Err(e)) => {
                    error!("Invalid message received: {:?}", anyhow!(e));
                    continue;
                },
                Delivery::Stop => break Ok(StopType::Stopped),
            };

            trace!("Worker {}: {:?}", worker_id, msg);

            match process(msg).await {
                Ok(()) => acker
                    .ack(BasicAckOptions::default())
                    .await
                    .context("Failed to send ACK for delivery")?,
                Err(e) => {
                    warn!("Failed to process {}: {:?}", e.1, e.0);

                    acker
                        .reject(BasicRejectOptions { requeue: false })
                        .await
                        .context("Failed to send NAK for delivery")?;
                },
            }
        }
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
        F: Send + Future<Output = MessageResult<D>> + 'static,
        D: Display + Send + 'static,
    >(
        params: &Params,
        conn: indexer_rabbitmq::lapin::Connection,
        consumer: Consumer<Q>,
        queue_type: Q,
        grace_period: StdDuration,
        process: impl Fn(Q::Message) -> F + Send + Sync + Clone + 'static,
    ) -> Result<()>
    where
        Q::Message: Debug + Send + for<'a> serde::Deserialize<'a>,
    {
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
                    Ok(Ok(StopType::Hangup)) => warn!("AMQP server hung up!"),
                    Ok(Ok(StopType::Stopped)) => (),
                    Ok(Err(e)) => error!("Fatal error in worker: {:?}", e),
                    Err(e) => error!("Worker terminated unexpectedly: {:?}", e),
                })
            })
            .collect::<FuturesUnordered<_>>();

        let signal;

        #[cfg(unix)]
        {
            use tokio::signal::unix::SignalKind;

            let mut stream = [
                SignalKind::hangup(),
                SignalKind::interrupt(),
                SignalKind::quit(),
                SignalKind::terminate(),
            ]
            .into_iter()
            .map(|k| {
                tokio::signal::unix::signal(k)
                    .with_context(|| format!("Failed to hook signal {k:?}"))
                    .map(|mut s| async move {
                        s.recv().await;
                        Result::<_>::Ok(k)
                    })
            })
            .collect::<Result<FuturesUnordered<_>>>()?;

            signal = async move { stream.next().await.transpose() }
        }

        #[cfg(not(unix))]
        {
            use std::fmt;

            use futures_util::TryFutureExt;

            struct CtrlC;

            impl fmt::Debug for CtrlC {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("^C")
                }
            }

            signal = tokio::signal::ctrl_c().map_ok(|()| Some(CtrlC));
        }

        let signal = tokio::select! {
            _ = q_tasks.next() => Ok(None),
            s = signal => s,
        }
        .context("Failed to wait for stop signal")?;

        //////// Everything past this point is graceful failure! ////////

        if let Some(signal) = signal {
            warn!("{:?} received, shutting down...", signal);
        } else {
            warn!("Worker terminated unexpectedly, shutting down...");
        }

        stop_tx.send(()).unwrap();
        dl_task.abort();

        if !q_tasks.is_empty() {
            info!("Waiting for additional jobs to finish...");
        }

        while let Some(()) = tokio::select! {
            t = q_tasks.next() => t,
            _ = tokio::time::sleep(grace_period) => None,
        } {}

        std::mem::drop(stop_tx);

        // NB: this shouldn't need the grace period because we abort the task
        dl_task
            .await
            .map_err(|e| {
                if !e.is_cancelled() {
                    error!("DLX consumer cleanup failed: {:?}", e);
                }
            })
            .unwrap_or(());

        Ok(())
    }
}
