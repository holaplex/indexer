use holaplex_indexer::http::{Client, ClientArgs};
use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::http_indexer;

#[derive(Debug, clap::Args)]
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

    #[clap(flatten)]
    queue_suffix: indexer_rabbitmq::suffix::Suffix,

    #[clap(flatten)]
    client: ClientArgs,
}

fn main() {
    holaplex_indexer::run(|args: Args, params, db| async move {
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

async fn run<E: Send + holaplex_indexer::http::Process + 'static>(
    args: Args,
    params: holaplex_indexer::Params,
    db: holaplex_indexer::db::Pool,
) -> Result<()> {
    let Args {
        amqp_url,
        sender,
        entity: _,
        queue_suffix,
        client,
    } = args;

    let conn = holaplex_indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;
    let client = Client::new_rc(db, client).context("Failed to construct Client")?;

    let queue_type = http_indexer::QueueType::<E>::new(&sender, &queue_suffix)?;
    let consumer = http_indexer::Consumer::new(&conn, queue_type.clone(), "http-consumer")
        .await
        .context("Failed to create queue consumer")?;

    holaplex_indexer::amqp_consume(&params, conn, consumer, queue_type, move |m| {
        let client = client.clone();
        async move { m.process(&client).await }
    })
    .await
}
