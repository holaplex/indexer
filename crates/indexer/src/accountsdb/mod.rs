//! Support features for the `accountsdb` indexer

mod accounts;
mod client;
mod programs;

pub use client::Client;
use indexer_core::pubkeys;
pub(self) use indexer_rabbitmq::accountsdb::AccountUpdate;
use indexer_rabbitmq::accountsdb::Message;

use crate::prelude::*;

/// Process a message from an accountsdb RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message, client: &Client) -> Result<()> {
    match msg {
        Message::AccountUpdate(update) if update.owner == pubkeys::metadata() => {
            programs::metadata::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::auction() => {
            programs::auction::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::metaplex() => {
            programs::metaplex::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::auction_house() => {
            programs::auction_house::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::token() => {
            programs::token::process(client, update).await
        },
        Message::AccountUpdate(_) | Message::InstructionNotify { .. } => Ok(()),
    }
}
