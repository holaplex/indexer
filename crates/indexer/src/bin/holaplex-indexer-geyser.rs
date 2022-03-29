use std::{collections::HashSet, sync::Arc};

use holaplex_indexer::geyser::Client;
use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::{geyser, http_indexer};

#[derive(Debug, clap::Parser)]
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

    /// List of programs to ignore on starup
    /// e.g. [metadata, auction_house]
    /// metadata,auction_house
    #[clap(long, env)]
    ignore_on_startup: Option<Vec<String>>,

    /// An optional suffix for the AMQP queue ID
    ///
    /// For debug builds a value must be provided here to avoid interfering with
    /// the indexer.
    queue_suffix: Option<String>,
}

fn main() {
    holaplex_indexer::run(
        |Args {
             amqp_url,
             network,
             startup,
             ignore_on_startup,
             queue_suffix,
         },
         params,
         db| async move {
            if cfg!(debug_assertions) && queue_suffix.is_none() {
                bail!("Debug builds must specify a RabbitMQ queue suffix!");
            }

            let sender = queue_suffix.clone().unwrap_or_else(|| network.to_string());

            let conn = holaplex_indexer::amqp_connect(amqp_url).await?;
            let client = Client::new_rc(
                db,
                &conn,
                http_indexer::QueueType::new(&sender, queue_suffix.as_deref()),
                http_indexer::QueueType::new(&sender, queue_suffix.as_deref()),
            )
            .await
            .context("Failed to construct Client")?;

            let queue_type = geyser::QueueType::new(network, startup, queue_suffix.as_deref());
            let consumer = geyser::Consumer::new(&conn, queue_type.clone())
                .await
                .context("Failed to create queue consumer")?;

            let ignore_on_startup = Arc::new(
                ignore_on_startup
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .collect::<HashSet<_>>(),
            );

            holaplex_indexer::amqp_consume(&params, conn, consumer, queue_type, move |m| {
                let client = client.clone();

                async move {
                    holaplex_indexer::geyser::process_message(m, &*client, ignore_on_startup).await
                }
            })
            .await
        },
    );
}
