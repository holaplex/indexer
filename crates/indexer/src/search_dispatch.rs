use indexer_rabbitmq::search_indexer::{Document, Message, Producer, QueueType};
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Serialize)]
pub struct TwitterHandleDocument {
    pub owner: String,
    pub handle: String,
}

#[derive(Debug, Serialize)]
pub struct MetadataDocument {
    pub image: String,
    #[serde(flatten)]
    pub raw: serde_json::Value,
}

#[derive(Debug)]
pub struct Client {
    producer: Producer,
}

impl Client {
    pub async fn new(conn: &indexer_rabbitmq::lapin::Connection, queue: QueueType) -> Result<Self> {
        Ok(Self {
            producer: Producer::new(conn, queue)
                .await
                .context("Couldn't create AMQP search producer")?,
        })
    }

    #[inline]
    async fn dispatch_upsert(
        &self,
        index: &'static str,
        id: impl std::fmt::Display,
        body: impl Serialize,
    ) -> Result<()> {
        self.producer
            .write(Message::Upsert {
                index: index.to_owned(),
                document: Document {
                    id: id.to_string(),
                    body: serde_json::to_value(body).context("Failed to upcast document body")?,
                },
            })
            .await
            .context("Failed to send upsert message")
    }

    pub async fn upsert_metadata(&self, key: String, body: MetadataDocument) -> Result<()> {
        debug_assert!(key.parse::<Pubkey>().is_ok());

        self.dispatch_upsert("metadatas", key, body).await
    }

    pub async fn upsert_twitter_handle(
        &self,
        key: Pubkey,
        body: TwitterHandleDocument,
    ) -> Result<()> {
        self.dispatch_upsert("name_service", key, body).await
    }
}
