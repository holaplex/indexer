use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::{accountsdb, http_indexer, prelude::*};
use metaplex_indexer::accountsdb::Client;

#[derive(Debug, clap::Parser)]
struct Args {
    /// The address of an AMQP server to connect to
    #[clap(long, env)]
    amqp_url: String,

    /// The network to listen to events for
    #[clap(long, env)]
    network: accountsdb::Network,

    /// The startup type of events to listen for
    #[clap(long, env, default_value_t = accountsdb::StartupType::Normal)]
    startup: accountsdb::StartupType,

    /// An optional suffix for the AMQP queue ID
    ///
    /// For debug builds a value must be provided here to avoid interfering with
    /// the indexer.
    queue_suffix: Option<String>,
}

fn main() {
    metaplex_indexer::run(
        |Args {
             amqp_url,
             network,
             startup,
             queue_suffix,
         },
         params,
         db| async move {
            if cfg!(debug_assertions) && queue_suffix.is_none() {
                bail!("Debug builds must specify a RabbitMQ queue suffix!");
            }

            let sender = queue_suffix.clone().unwrap_or_else(|| network.to_string());

            let conn = metaplex_indexer::amqp_connect(amqp_url).await?;
            let client = Client::new_rc(
                db,
                &conn,
                http_indexer::QueueType::new(&sender, queue_suffix.as_deref()),
                http_indexer::QueueType::new(&sender, queue_suffix.as_deref()),
            )
            .await
            .context("Failed to construct Client")?;

            let queue_type = accountsdb::QueueType::new(network, startup, queue_suffix.as_deref());
            let consumer = queue_type
                .clone()
                .consumer(&conn)
                .await
                .context("Failed to create queue consumer")?;

            metaplex_indexer::amqp_consume(&params, conn, consumer, queue_type, move |m| {
                let client = client.clone();
                async move { metaplex_indexer::accountsdb::process_message(m, &*client).await }
            })
            .await
        },
    );
}
