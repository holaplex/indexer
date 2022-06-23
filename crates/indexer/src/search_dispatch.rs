use indexer_core::clap;
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
    pub name: String,
    pub mint_address: String,
    pub image: Option<String>,
    pub creator_address: String,
    pub creator_twitter_handle: Option<String>,
}

#[derive(Debug, Clone, Copy, clap::Parser)]
pub struct Args {
    /// Pass this flag to run backfill search upsert jobs
    ///
    /// Be aware that this can have severe performance implications.
    #[clap(long, env)]
    backfill_search: bool,
}

#[derive(Debug)]
pub struct Client {
    producer: Producer,
    backfill: bool,
}

impl Client {
    pub async fn new(
        conn: &indexer_rabbitmq::lapin::Connection,
        queue: QueueType,
        Args { backfill_search }: Args,
    ) -> Result<Self> {
        Ok(Self {
            producer: Producer::new(conn, queue)
                .await
                .context("Couldn't create AMQP search producer")?,
            backfill: backfill_search,
        })
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
