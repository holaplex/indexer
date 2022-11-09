use indexer_core::{clap, prelude::*};
use indexer_rabbitmq::job_runner;

/// Indexer worker for running scheduled jobs
#[derive(Debug, clap::Args)]
#[group(skip)]
#[command(name = "holaplex-indexer-job-runner", version, long_about = None)]
struct Args {
    /// The address of an AMQP server to connect to
    #[arg(long, env)]
    amqp_url: String,

    /// The ID of the indexer sending events to listen for
    #[arg(long, env)]
    sender: String,

    #[command(flatten)]
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

            let queue_type = job_runner::QueueType::new(&sender, &queue_suffix)?;
            let consumer = job_runner::Consumer::new(&conn, queue_type.clone(), "job-consumer")
                .await
                .context("Failed to create queue consumer")?;

            holaplex_indexer::amqp_consume(
                &params,
                conn,
                consumer,
                queue_type,
                StdDuration::from_secs(120),
                move |m| async move { holaplex_indexer::jobs::process_message(m).await },
            )
            .await
        },
    )
}
