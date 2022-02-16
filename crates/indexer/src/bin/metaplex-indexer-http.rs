use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::http_indexer;

#[derive(Debug, clap::Parser)]
struct Args {
    /// The address of an AMQP server to connect to
    #[clap(long, env)]
    amqp_url: String,

    /// The ID of the indexer sending events to listen for
    #[clap(long, env)]
    sender: String,

    /// The entity type to listen to events for
    #[clap(long, env)]
    entity: http_indexer::EntityId,

    /// An optional suffix for the AMQP queue ID
    ///
    /// For debug builds a value must be provided here to avoid interfering with
    /// the indexer.
    queue_suffix: Option<String>,

    /// A valid base URL to use when fetching IPFS links
    #[clap(long, env)]
    ipfs_cdn: Option<String>,

    /// A valid base URL to use when fetching Arweave links
    #[clap(long, env)]
    arweave_cdn: Option<String>,
}

fn main() {
    metaplex_indexer::run(|args: Args, db| async move {
        use http_indexer::{EntityId, MetadataJson, StoreConfig};

        match args.entity {
            EntityId::MetadataJson => run::<MetadataJson>(args, db).await,
            EntityId::StoreConfig => run::<StoreConfig>(args, db).await,
        }
    });
}

async fn run<E: http_indexer::Entity>(args: Args, db: indexer_core::db::Pool) -> Result<()> {
    let Args {
        amqp_url,
        sender,
        entity: _,
        queue_suffix,
        ipfs_cdn,
        arweave_cdn,
    } = args;

    let mut consumer = metaplex_indexer::create_consumer(
        amqp_url,
        http_indexer::QueueType::<E>::new(&sender, queue_suffix.as_deref()),
    )
    .await?;

    while let Some(msg) = consumer
        .read()
        .await
        .context("Failed to read message from RabbitMQ")?
    {
        trace!("{:?}", msg);

        match metaplex_indexer::http::process_message(msg).await {
            Ok(()) => (),
            Err(e) => error!("Failed to process message: {:?}", e),
        }
    }

    warn!("AMQP server hung up!");

    Ok(())
}
