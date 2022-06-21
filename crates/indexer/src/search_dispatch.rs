use indexer_core::{clap, meilisearch};
use indexer_rabbitmq::search_indexer::{Document, Message, Producer, QueueType};
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Serialize)]
pub struct TwitterHandleDocument {
    pub owner: String,
    pub handle: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MetadataDocument {
    pub name: String,
    pub mint_address: String,
    pub image: Option<String>,
    pub creator_address: String,
    pub creator_twitter_handle: Option<String>,
    pub collection_address: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CollectionDocument {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub mint_address: String,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    /// Pass this flag to run backfill search upsert jobs
    ///
    /// Be aware that this can have severe performance implications.
    #[clap(long, env)]
    backfill_search: bool,

    #[clap(flatten)]
    search: meilisearch::Args,
}

#[derive(Debug)]
pub struct Client {
    producer: Producer,
    backfill: bool,
    meili_client: meilisearch::client::Client,
}

impl Client {
    pub async fn new(
        conn: &indexer_rabbitmq::lapin::Connection,
        queue: QueueType,
        Args {
            backfill_search,
            search,
        }: Args,
    ) -> Result<Self> {
        Ok(Self {
            producer: Producer::new(conn, queue)
                .await
                .context("Couldn't create AMQP search producer")?,
            backfill: backfill_search,
            meili_client: search.into_client(),
        })
    }

    // Gets a document using the id
    ///
    /// # Errors
    /// This function fails if the index or document from the meilisearch client can not be fetched
    pub async fn get_document(&self, idx: String, id: String) -> Result<serde_json::Value> {
        let index = self
            .meili_client
            .get_index(idx)
            .await
            .context("failed to get index")?;

        let document = index
            .get_document::<serde_json::Value>(&id)
            .await
            .context("failed to get document")?;
        Ok(document)
    }

    #[inline]
    async fn dispatch_upsert(
        &self,
        is_for_backfill: bool,
        index: &'static str,
        id: impl std::fmt::Display,
        body: impl Serialize,
    ) -> Result<()> {
        if is_for_backfill && !self.backfill {
            return Ok(());
        }

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

    pub async fn upsert_metadata(
        &self,
        is_for_backfill: bool,
        key: String,
        body: MetadataDocument,
    ) -> Result<()> {
        debug_assert!(key.parse::<Pubkey>().is_ok());

        self.dispatch_upsert(is_for_backfill, "metadatas", key, body)
            .await
    }

    pub async fn upsert_collection(
        &self,
        is_for_backfill: bool,
        key: String,
        body: CollectionDocument,
    ) -> Result<()> {
        debug_assert!(key.parse::<Pubkey>().is_ok());

        self.dispatch_upsert(is_for_backfill, "collections", key, body)
            .await
    }

    pub async fn upsert_twitter_handle(
        &self,
        is_for_backfill: bool,
        key: Pubkey,
        body: TwitterHandleDocument,
    ) -> Result<()> {
        self.dispatch_upsert(is_for_backfill, "name_service", key, body)
            .await
    }
}
