//! Support features for the job runner

use std::fmt;

pub use client::{Args as ClientArgs, Client};
use indexer_rabbitmq::job_runner::{Message, SlotReindex};

use crate::prelude::*;

mod client;
mod slot_reindex;

/// Message identifier
#[derive(Debug, Clone)]
pub enum MessageId {
    /// A refresh of a cache table
    RefreshTable(String),
    /// A reindex of a slot
    ReindexSlot(SlotReindex),
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefreshTable(n) => write!(f, "refresh of cached table {n}"),
            Self::ReindexSlot(SlotReindex { slot, startup }) => {
                write!(f, "reindex of slot {slot} for startup type {startup}")
            },
        }
    }
}

/// Process a message from the background job RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(client: &Client, msg: Message) -> MessageResult<MessageId> {
    let id = match msg {
        Message::RefreshTable(ref n) => MessageId::RefreshTable(n.clone()),
        Message::ReindexSlot(s) => MessageId::ReindexSlot(s),
    };

    match msg {
        Message::RefreshTable(n) => process_refresh(n).await,
        Message::ReindexSlot(s) => slot_reindex::process(client, s).await,
    }
    .map_err(|e| MessageError::new(e, id))
}

#[allow(unreachable_code)]
async fn process_refresh(name: String) -> Result<()> {
    debug!("Refreshing table {:?}", name);

    todo!("Not yet implemented!");

    std::future::ready(Ok(())).await
}
