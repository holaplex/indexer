use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::{http_indexer, prelude::*};
use metaplex_indexer::http::Client;

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
    ipfs_cdn: String,

    /// A valid base URL to use when fetching Arweave links
    #[clap(long, env)]
    arweave_cdn: String,
}

fn main() {
    metaplex_indexer::run(|args: Args, params, db| async move {
        use http_indexer::{EntityId, MetadataJson, StoreConfig};

        // Note: each match arm will increase the compiled size of this
        //       binary, it may be advantageous to split this into separate
        //       binaries at some point.
        match args.entity {
            EntityId::MetadataJson => run::<MetadataJson>(args, params, db).await,
            EntityId::StoreConfig => run::<StoreConfig>(args, params, db).await,
        }
    });
}

async fn run<E: Send + metaplex_indexer::http::Process + 'static>(
    args: Args,
    params: metaplex_indexer::Params,
    db: metaplex_indexer::db::Pool,
) -> Result<()> {
    let Args {
        amqp_url,
        sender,
        entity: _,
        queue_suffix,
        ipfs_cdn,
        arweave_cdn,
    } = args;

    let conn = metaplex_indexer::amqp_connect(amqp_url).await?;
    let client = Client::new_rc(
        db,
        ipfs_cdn.parse().context("Failed to parse IPFS CDN URL")?,
        arweave_cdn
            .parse()
            .context("Failed to parse Arweave CDN URL")?,
    )
    .context("Failed to construct Client")?;

    let queue_type = http_indexer::QueueType::<E>::new(&sender, queue_suffix.as_deref());
    let consumer = queue_type
        .clone()
        .consumer(&conn)
        .await
        .context("Failed to create queue consumer")?;

    metaplex_indexer::amqp_consume(&params, conn, consumer, queue_type, move |m| {
        let client = client.clone();
        async move { m.process(&client).await }
    })
    .await
}
