use anyhow::Context;
use docbot::prelude::*;
use indexer_rabbitmq::{
    accountsdb::{Network, QueueType, StartupType},
    lapin::{Connection, ConnectionProperties},
    prelude::*,
};
use lapinou::LapinSmolExt;

#[derive(Docbot)]
pub enum RmqCommand {
    /// `listen <address> <network> <startup> <suffix>`
    /// Open an AMQP connection to the specified address
    ///
    /// # Arguments
    /// address: The address to connect to
    /// network: The network identifier of the server to listen for
    /// startup: The startup-type identifier of the server to listen for
    /// suffix: A unique identifier to suffix a new queue with
    Listen(String, Network, StartupType, String),
}

pub fn handle(cmd: RmqCommand) -> super::Result {
    match cmd {
        RmqCommand::Listen(addr, network, startup, suffix) => smol::block_on(async {
            let conn = Connection::connect(&addr, ConnectionProperties::default().with_smol())
                .await
                .context("Failed to connect to the AMQP server")?;
            let mut consumer = QueueType::new(network, startup, Some(&suffix))
                .consumer(&conn)
                .await
                .context("Failed to create a consumer")?;

            while let Some(msg) = consumer
                .read()
                .await
                .context("Failed to receive a message")?
            {
                eprintln!("Got message: {:?}", msg);
            }

            Ok(())
        }),
    }
}
