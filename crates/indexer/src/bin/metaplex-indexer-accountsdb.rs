use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::accountsdb;

#[derive(Debug, clap::Parser)]
struct Args {
    /// The address of an AMQP server to connect to
    #[clap(long, env)]
    amqp_url: String,

    /// The network to listen to events for
    #[clap(long, env)]
    network: accountsdb::Network,

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
             queue_suffix,
         },
         client| async move {
            if cfg!(debug_assertions) && queue_suffix.is_none() {
                bail!("Debug builds must specify a RabbitMQ queue suffix!");
            }

            let mut consumer = metaplex_indexer::create_consumer(
                amqp_url,
                accountsdb::QueueType::new(network, queue_suffix.as_deref()),
            )
            .await?;

            while let Some(msg) = consumer
                .read()
                .await
                .context("Failed to read message from RabbitMQ")?
            {
                trace!("{:?}", msg);

                match metaplex_indexer::accountsdb::process_message(msg, &*client).await {
                    Ok(()) => (),
                    Err(e) => error!("Failed to process message: {:?}", e),
                }
            }

            warn!("AMQP server hung up!");

            Ok(())
        },
    );
}
