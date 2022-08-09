use indexer_core::{clap, meilisearch};
use indexer_rabbitmq::search_indexer::{Document, Message, Producer, QueueType};
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Serialize)]
pub struct TwitterHandleDocument {
    pub owner: String,
    pub handle: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionDocument {
    pub name: String,
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

    /// Gets a document using the id
    ///
    /// # Errors
    /// This function fails if the index or document from the meilisearch client can not be fetched
    pub async fn get_document(
        &self,
        idx: String,
        id: String,
    ) -> Result<serde_json::Value, meilisearch::errors::Error> {
        let index = self.meili_client.get_index(idx).await?;

        index.get_document::<serde_json::Value>(&id).await
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

    #[inline]
    async fn dispatch_indirect_meta(
        &self,
        is_for_backfill: bool,
        index: &'static str,
        mint: Pubkey,
    ) -> Result<()> {
        if is_for_backfill && !self.backfill {
            return Ok(());
        }

        self.producer
            .write(Message::IndirectMetadata {
                index: index.to_owned(),
                mint,
            })
            .await
            .context("Failed to send indirect metadata message")
    }

    pub async fn upsert_geno_habitat(&self, is_for_backfill: bool, key: Pubkey) -> Result<()> {
        self.dispatch_indirect_meta(is_for_backfill, "geno_habitats", key)
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
