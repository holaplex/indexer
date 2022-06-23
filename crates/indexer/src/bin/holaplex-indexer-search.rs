use holaplex_indexer::search::{Client, ClientArgs};
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

    #[clap(flatten)]
    client: ClientArgs,
}

fn main() {
    holaplex_indexer::run(
        |Args {
             amqp_url,
             sender,
             queue_suffix,
             client,
         },
         params,
         db| async move {
            let conn = holaplex_indexer::amqp_connect(amqp_url, env!("CARGO_BIN_NAME")).await?;

            let queue_type = search_indexer::QueueType::new(&sender, &queue_suffix)?;
            let consumer =
                search_indexer::Consumer::new(&conn, queue_type.clone(), "search-consumer")
                    .await
                    .context("Failed to create queue consumer")?;

            let (client, upsert_task, stop_upsert) = Client::new_rc(db, client)
                .await
                .context("Failed to construct Client")?;

            let ret = holaplex_indexer::amqp_consume(
                &params,
                conn,
                consumer,
                queue_type,
                StdDuration::from_millis(500),
                move |m| {
                    let client = client.clone();
                    async move { holaplex_indexer::search::process_message(m, &*client).await }
                },
            )
            .await;

            if let Err(()) = stop_upsert.send(()) {
                error!("Failed to stop upsert task");
                upsert_task.abort();
            }

            if let Err(e) = tokio::select! {
                r = upsert_task => r.context("Join for upsert task failed"),
                _ = tokio::time::sleep(
                    StdDuration::from_secs(
                        if cfg!(debug_assertions) {
                            5
                        } else {
                            30
                        }
                    )
                ) => {
                    Err(anyhow!("Timed out waiting for upsert worker"))
                },
            } {
                error!("{:?}", e);
            }

            ret
        },
    );
}
