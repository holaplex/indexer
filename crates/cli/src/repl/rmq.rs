use anyhow::Context;
use docbot::prelude::*;
use indexer_rabbitmq::{
    accountsdb::{Network, QueueType},
    lapin::{Connection, ConnectionProperties},
    prelude::*,
};
use lapinou::LapinSmolExt;

#[derive(Docbot)]
pub enum RmqCommand {
    /// `listen <network> <address>`
    /// Open an AMQP connection to the specified address
    ///
    /// # Arguments
    /// network: The network identifier of the server to listen for
    /// address: The address to connect to
    Listen(Network, String),
}

pub fn handle(cmd: RmqCommand) -> super::Result {
    match cmd {
        RmqCommand::Listen(network, addr) => smol::block_on(async {
            let conn = Connection::connect(&addr, ConnectionProperties::default().with_smol())
                .await
                .context("Failed to connect to the AMQP server")?;
            let mut consumer = QueueType::new(network)
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
