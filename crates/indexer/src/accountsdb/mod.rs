//! Support features for the `accountsdb` indexer

mod accounts;
mod client;
// mod get_storefronts;
mod programs;
// mod store_owner;

pub use client::Client;
use indexer_core::pubkeys;
use indexer_rabbitmq::accountsdb::Message;

use crate::prelude::*;

/// Process a message from an accountsdb RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message, client: &Client) -> Result<()> {
    match msg {
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::metadata() => {
            programs::metadata::process(client, key, data).await
        },
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::auction() => {
            programs::auction::process(client, key, data).await
        },
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::metaplex() => {
            programs::metaplex::process(client, key, data).await
        },
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::auction_house() => {
            programs::auction_house::process(client, key, data).await
        },
        Message::AccountUpdate { owner, key, data } if owner == pubkeys::token() => {
            programs::token::process(client, key, data).await
        },
        Message::AccountUpdate { .. } | Message::InstructionNotify { .. } => Ok(()),
    }
}
