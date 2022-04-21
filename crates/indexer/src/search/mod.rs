//! Support features for the search indexer

mod client;

pub use client::{Args as ClientArgs, Client};
use indexer_rabbitmq::search_indexer::{self, Message};

use crate::prelude::*;

/// A schemaless Meilisearch document
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document<T: serde::Serialize> {
    id: String,
    #[serde(flatten)]
    body: T,
}

impl<T: serde::Serialize> From<search_indexer::Document<T>> for Document<T> {
    fn from(search_indexer::Document { id, body }: search_indexer::Document<T>) -> Self {
        Self { id, body }
    }
}

impl<T: serde::Serialize + std::fmt::Debug + serde::de::DeserializeOwned>
    meilisearch_sdk::document::Document for Document<T>
{
    type UIDType = String;

    fn get_uid(&self) -> &String {
        &self.id
    }
}

/// Process a message from a search RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message, client: &Client) -> Result<()> {
    match msg {
        Message::Upsert { index, document } => {
            client.upsert_documents(index, &[document.into()]).await?;

            Ok(())
        },
    }
}
