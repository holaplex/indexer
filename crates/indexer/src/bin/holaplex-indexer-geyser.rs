use std::{collections::HashSet, sync::Arc};

use holaplex_indexer::geyser::{Client, ClientArgs, IgnoreType};
use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::{geyser, http_indexer, suffix::Suffix};

#[derive(Debug, clap::Args)]
struct Args {
    /// The address of an AMQP server to connect to
    #[clap(long, env)]
    amqp_url: String,

    /// The network to listen to events for
    #[clap(long, env)]
    network: geyser::Network,

    /// The startup type of events to listen for
    #[clap(long, env, default_value_t = geyser::StartupType::Normal)]
    startup: geyser::StartupType,

    /// List of topics or programs to ignore on startup
    ///
    /// For example, `metadata,candy-machine` will ignore the Metaplex metadata
    /// and candy machine programs.
    #[clap(long, env, use_value_delimiter(true))]
    ignore_on_startup: Option<Vec<IgnoreType>>,

    #[clap(flatten)]
    queue_suffix: Suffix,

    #[clap(flatten)]
    client: ClientArgs,
}

fn main() {
    holaplex_indexer::run(
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
            let sender = match queue_suffix {
                Suffix::Debug(ref s) => s.clone(),
                _ => network.to_string(),
            };

            let conn = holaplex_indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;
            let client = Client::new_rc(
                db,
                &conn,
                http_indexer::QueueType::new(&sender, &queue_suffix)?,
                http_indexer::QueueType::new(&sender, &queue_suffix)?,
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

            holaplex_indexer::amqp_consume(&params, conn, consumer, queue_type, move |m| {
                let client = client.clone();
                let ignore_on_startup = ignore_on_startup.clone();

                async move {
                    holaplex_indexer::geyser::process_message(m, &*client, ignore_on_startup).await
                }
            })
            .await
        },
    );
}
