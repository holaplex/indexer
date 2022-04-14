use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::search_indexer;

#[derive(Debug, clap::Parser)]
struct Args {
    /// The address of an AMQP server to connect to
    #[clap(long, env)]
    amqp_url: String,

    /// The ID of the indexer sending events to listen for
    #[clap(long, env)]
    sender: String,

    #[clap(flatten)]
    queue_suffix: indexer_rabbitmq::suffix::Suffix,
}

fn main() {
    holaplex_indexer::run(
        |Args {
             amqp_url,
             sender,
             queue_suffix,
         },
         params,
         _db| async move {
            let conn = holaplex_indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;

            let queue_type = search_indexer::QueueType::new(&sender, &queue_suffix)?;
            let consumer =
                search_indexer::Consumer::new(&conn, queue_type.clone(), "search-consumer")
                    .await
                    .context("Failed to create queue consumer")?;

            holaplex_indexer::amqp_consume(
                &params,
                conn,
                consumer,
                queue_type,
                move |m| async move { process(m).await },
            )
            .await
        },
    );
}

async fn process(m: search_indexer::Message) -> Result<()> {
    debug!("TODO: {:?}", m);

    Ok(())
}
