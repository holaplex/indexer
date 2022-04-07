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

/// A value indicating a specific topic to ignore
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum IgnoreType {
    /// Ignore the metadata program
    Metadata,
    /// Ignore the Metaplex candy machine program
    CandyMachine,
    /// Ignore the SPL token program
    Tokens,
}

/// Process a message from a Geyser RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message<H: std::hash::BuildHasher>(
    msg: Message,
    client: &Client,
    ignore_on_startup: Arc<HashSet<IgnoreType, H>>,
) -> Result<()> {
    let check_ignore =
        |ty, update: &AccountUpdate| !(update.is_startup && ignore_on_startup.contains(&ty));

    match msg {
        // Message::AccountUpdate(update)
        //     if update.owner == pubkeys::metadata()
        //         && check_ignore(IgnoreType::Metadata, &update) =>
        // {
        //     programs::metadata::process(client, update).await
        // },
        // Message::AccountUpdate(update) if update.owner == pubkeys::auction() => {
        //     programs::auction::process(client, update).await
        // },
        // Message::AccountUpdate(update) if update.owner == pubkeys::metaplex() => {
        //     programs::metaplex::process(client, update).await
        // },
        // Message::AccountUpdate(update) if update.owner == pubkeys::auction_house() => {
        //     programs::auction_house::process(client, update).await
        // },
        // Message::AccountUpdate(update)
        //     if update.owner == pubkeys::token() && check_ignore(IgnoreType::Tokens, &update) =>
        // {
        //     programs::token::process(client, update).await
        // },
        // Message::AccountUpdate(update) if update.owner == pubkeys::graph_program() => {
        //     programs::graph::process(client, update).await
        // },
        // Message::AccountUpdate(update)
        //     if update.owner == pubkeys::candy_machine()
        //         && check_ignore(IgnoreType::CandyMachine, &update) =>
        // {
        //     programs::candy_machine::process(client, update).await
        // },
        // Message::AccountUpdate(update) if update.owner == pubkeys::name_service() => {
        //     programs::name_service::process(client, update).await
        // },
        Message::AccountUpdate(update) if update.owner == pubkeys::cardinal_token_manager() => {
            programs::cardinal_token_manager::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::cardinal_time_invalidator() => {
            programs::cardinal_time_invalidator::process(client, update).await
        },
        Message::AccountUpdate(update) if update.owner == pubkeys::cardinal_use_invalidator() => {
            programs::cardinal_use_invalidator::process(client, update).await
        },
        Message::AccountUpdate(update)
            if update.owner == pubkeys::cardinal_paid_claim_approver() =>
        {
            programs::cardinal_paid_claim_approver::process(client, update).await
        },
        Message::AccountUpdate(update) => {
            // debug!(
            //     "Unhandled account update for program {}",
            //     bs58::encode(update.owner).into_string()
            // );
            Ok(())
        },
        Message::InstructionNotify { .. } => Ok(()),
    }
}
