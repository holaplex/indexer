use indexer_core::{clap, meilisearch};
use indexer_rabbitmq::search_indexer::{Document, Message, Producer, QueueType};
use serde::Serialize;

use crate::prelude::*;
#[allow(missing_docs)]
/// `MeiliSearch` document for name-services index
#[derive(Debug, Serialize)]
pub struct TwitterHandleDocument {
    pub owner: String,
    pub handle: String,
}

#[allow(missing_docs)]
/// `Meilisearch` document for 'collections' index
#[derive(Debug, Clone, Serialize)]
pub struct CollectionDocument {
    pub name: String,
    pub image: Option<String>,
    pub mint_address: String,
}

#[allow(missing_docs)]
/// ``Meilisearch`` document for 'mr-collections' index
#[derive(Debug, Clone, Serialize)]
pub struct MRCollectionDocument {
    pub name: String,
    pub image: Option<String>,
    pub magic_eden_id: Option<String>,
    pub verified_collection_address: Option<String>,
    pub twitter_url: Option<String>,
    pub discord_url: Option<String>,
    pub website_url: Option<String>,
}

/// Arguments to build the ``search_dispatch`` client
#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    /// Pass this flag to run backfill search upsert jobs
    ///
    /// Be aware that this can have severe performance implications.
    #[clap(long, env)]
    backfill_search: bool,

    /// Meilisearch arguments
    /// Contains Key and URL
    #[clap(flatten)]
    search: meilisearch::Args,
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Client {
    producer: Producer,
    backfill: bool,
    meili_client: meilisearch::client::Client,
}

#[allow(dead_code)]
impl Client {
    /// Creates a ``search_dispatch`` client
    ///
    /// # Errors
    /// This function fails if it fails to construct meilisearch client or rabbitmq producer
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

    /// Dispatches geno habitat document message to the AMQP queue
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn upsert_geno_habitat(&self, is_for_backfill: bool, mint: Pubkey) -> Result<()> {
        self.dispatch_indirect_meta(is_for_backfill, "geno_habitats", mint)
            .await
    }

    /// Dispatches collection document message to the AMQP queue
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
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

    /// Dispatches moonrank collection document message to the AMQP queue
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
    pub async fn upsert_mr_collection(
        &self,
        is_for_backfill: bool,
        key: String,
        body: MRCollectionDocument,
    ) -> Result<()> {
        self.dispatch_upsert(is_for_backfill, "mr-collections", key, body)
            .await
    }

    /// Dispatches twitter name service message to the AMQP queue
    ///
    /// # Errors
    /// This function fails if the AMQP payload cannot be sent.
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
