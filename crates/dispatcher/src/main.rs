//! Utility for manually dispatching indexer job messages

#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use indexer_core::{clap, clap::Parser, prelude::*};
use indexer_rabbitmq::{
    job_runner::{self, Message},
    lapin,
};

#[derive(Debug, Parser)]
#[command(about, version, long_about = None)]
struct Opts {
    /// The address of an AMQP server to connect to
    #[arg(long, env)]
    amqp_url: String,

    /// The ID of the indexer sending events to listen for
    #[arg(long, env)]
    sender: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Request a refresh of a table of cached data
    RefreshTable {
        /// The name of the table to request a data refresh for
        #[arg(env)]
        name: String,
    },
    /// Request a re-indexing of transactions from a block
    ReindexBlock {
        /// The slot number of the block to reindex
        #[arg(env)]
        slot: u64,
    },
}

fn main() {
    indexer_core::run(|| {
        let exec = smol::LocalExecutor::new();

        smol::block_on(exec.run(async {
            let opts = Opts::parse();
            debug!("{:#?}", opts);

            let Opts {
                amqp_url,
                sender,
                cmd,
            } = opts;

            let conn = lapin::Connection::connect(
                &amqp_url,
                lapin::ConnectionProperties::default()
                    .with_connection_name(
                        format!(
                            "dispatcher@{}",
                            hostname::get()
                                .context("Failed to get system hostname")?
                                .into_string()
                                .map_err(|_| anyhow!("Failed to parse system hostname"))?,
                        )
                        .into(),
                    )
                    .with_executor(smol_executor_trait::Smol)
                    .with_reactor(async_reactor_trait::AsyncIo),
            )
            .await
            .context("Failed to connect to the AMQP server")?;

            let queue_type = job_runner::QueueType::new(
                &sender,
                &indexer_rabbitmq::suffix::Suffix::ProductionUnchecked,
            )?;
            let producer = job_runner::Producer::new(&conn, queue_type)
                .await
                .context("Failed to create message producer")?;

            match cmd {
                Command::RefreshTable { name } => producer.write(Message::RefreshTable(name)).await,
                Command::ReindexBlock { slot } => producer.write(Message::ReindexSlot(slot)).await,
            }
            .context("Failed to send requested message")?;

            Ok(())
        }))
    })
}
