//! Support features for the search indexer

mod client;

use std::fmt;

pub use client::{Args as ClientArgs, Client};
use indexer_rabbitmq::search_indexer::{self, Message};

use crate::prelude::*;

/// Message identifier
#[derive(Debug, Clone, Copy)]
pub struct MessageId;

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "document upsert")
    }
}

/// A schemaless Meilisearch document
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
    id: String,
    #[serde(flatten)]
    body: serde_json::Value,
}

impl From<search_indexer::Document> for Document {
    fn from(search_indexer::Document { id, body }: search_indexer::Document) -> Self {
        Self { id, body }
    }
}

/// Process a message from a search RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message, client: &Client) -> MessageResult<MessageId> {
    match msg {
        Message::Upsert { index, document } => {
            client
                .upsert_documents(index, Some(document.into()))
                .await
                .map_err(|e| MessageError::new(e, MessageId))?;

            Ok(())
        },
    }
}
