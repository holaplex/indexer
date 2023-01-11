use std::{collections::HashSet, sync::Arc};

mod handler;

use handler::{Client, ClientArgs, ClientQueues, IgnoreType};
use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::{geyser, http_indexer, job_runner, search_indexer, suffix::Suffix};

/// Indexer worker for receiving Geyser messages
#[derive(Debug, clap::Args)]
#[group(skip)]
#[command(name = "holaplex-indexer-geyser", version, long_about = None)]
struct Args {
    /// The address of an AMQP server to connect to
    #[arg(long, env)]
    amqp_url: String,

    /// The network to listen to events for
    #[arg(long, env)]
    network: geyser::Network,

    /// The startup type of events to listen for
    #[arg(long, env, default_value_t = geyser::StartupType::Normal)]
    startup: geyser::StartupType,

    /// List of topics or programs to ignore on startup
    ///
    /// For example, `metadata,candy-machine` will ignore the Metaplex metadata
    /// and candy machine programs.
    #[arg(long, env, use_value_delimiter(true))]
    ignore_on_startup: Option<Vec<IgnoreType>>,

    #[command(flatten)]
    queue_suffix: indexer_core::queue_suffix::QueueSuffix,

    #[command(flatten)]
    client: ClientArgs,
}

fn main() {
    indexer::run(
        |Args {
             amqp_url,
             network,
             startup,
             ignore_on_startup,
             queue_suffix,
             client,
         },
         params,
         db| async move {
            let queue_suffix = queue_suffix.into();
            let receiver = match queue_suffix {
                Suffix::Debug(ref s) => s.clone(),
                _ => network.to_string(),
            };

            let conn = indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;
            let client = Client::new_rc(
                db,
                &conn,
                ClientQueues {
                    metadata_json: http_indexer::QueueType::new(&receiver, &queue_suffix)?,
                    store_config: http_indexer::QueueType::new(&receiver, &queue_suffix)?,
                    search: search_indexer::QueueType::new(&receiver, &queue_suffix)?,
                    jobs: job_runner::QueueType::new(&receiver, &queue_suffix)?,
                },
                startup,
                client,
            )
            .await
            .context("Failed to construct Client")?;

            let queue_type = geyser::QueueType::new(network, startup, &queue_suffix)?;
            let consumer = geyser::Consumer::new(&conn, queue_type.clone(), "geyser-consumer")
                .await
                .context("Failed to create queue consumer")?;

            let ignore_on_startup = Arc::new(
                ignore_on_startup
                    .into_iter()
                    .flatten()
                    .collect::<HashSet<_>>(),
            );

            indexer::amqp_consume(
                &params,
                conn,
                consumer,
                queue_type,
                StdDuration::from_millis(100),
                move |m| {
                    let client = client.clone();
                    let ignore_on_startup = ignore_on_startup.clone();

                    async move { handler::process_message(m, &client, ignore_on_startup).await }
                },
            )
            .await
        },
    );
}
