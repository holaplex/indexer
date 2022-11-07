//! Support features for the job runner

use std::fmt;

use indexer_rabbitmq::job_runner::Message;

use crate::prelude::*;

/// Message identifier
#[derive(Debug, Clone)]
pub enum MessageId {
    /// A refresh of a cache table
    RefreshTable(String),
    /// A reindex of a slot
    ReindexSlot(u64),
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefreshTable(n) => write!(f, "refresh of cached table {n}"),
            Self::ReindexSlot(s) => write!(f, "reindex of slot {s}"),
        }
    }
}

/// Process a message from the background job RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message) -> MessageResult<MessageId> {
    let id = match msg {
        Message::RefreshTable(ref n) => MessageId::RefreshTable(n.clone()),
        Message::ReindexSlot(i) => MessageId::ReindexSlot(i),
    };

    match msg {
        Message::RefreshTable(n) => process_refresh(n).await,
        Message::ReindexSlot(s) => process_reindex_slot(s).await,
    }
    .map_err(|e| MessageError::new(e, id))
}

#[allow(unreachable_code)]
async fn process_refresh(name: String) -> Result<()> {
    debug!("Refreshing table {:?}", name);

    todo!("Not yet implemented!");

    std::future::ready(Ok(())).await
}

async fn process_reindex_slot(id: u64) -> Result<()> {
    debug!("Reindexing slot {:?}", id);

    Ok(())
}
