//! Support features for the Geyser indexer

mod accounts;
mod client;
mod programs;

use std::{collections::HashSet, sync::Arc};

pub use client::Client;
use indexer_core::pubkeys;
pub(self) use indexer_rabbitmq::geyser::AccountUpdate;
use indexer_rabbitmq::geyser::Message;

use crate::prelude::*;

/// Process a message from a Geyser RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(
    msg: Message,
    client: &Client,
    ignore_on_startup: Arc<HashSet<String>>,
) -> Result<()> {
    let check_ignore =
        |string, update: &AccountUpdate| !(update.is_startup && ignore_on_startup.contains(string));

    match msg {
        Message::AccountUpdate(update)
            if update.owner == pubkeys::metadata() && check_ignore("metadata", &update) =>
        {
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
        Message::AccountUpdate(update) if update.owner == pubkeys::graph_program() => {
            programs::graph::process(client, update).await
        },
        Message::AccountUpdate(update)
            if update.owner == pubkeys::candy_machine()
                && check_ignore("candy_machine", &update) =>
        {
            programs::candy_machine::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::name_service() => {
            programs::name_service::process(client, update).await
        },
        Message::AccountUpdate(update) => {
            debug!("{:?}", update.owner);
            Ok(())
        },
        Message::InstructionNotify { .. } => Ok(()),
    }
}
