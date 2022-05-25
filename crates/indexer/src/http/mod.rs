//! Support features for the HTTP indexer

pub(self) mod client;
mod metadata_json;
mod store_config;

pub use client::{Args as ClientArgs, Client};
use indexer_rabbitmq::http_indexer::{Entity, MetadataJson, StoreConfig};

use crate::prelude::*;

/// Define processing logic for an incoming entity type
#[async_trait::async_trait]
pub trait Process: Entity {
    /// Process and consume an incoming entity
    async fn process(self, client: &Client) -> Result<()>;
}

#[async_trait::async_trait]
impl Process for MetadataJson {
    async fn process(self, client: &Client) -> Result<()> {
        let MetadataJson {
            meta_address,
            first_verified_creator,
            uri,
            slot_info,
        } = self;

        metadata_json::process(client, meta_address, first_verified_creator, uri, slot_info).await
    }
}

#[async_trait::async_trait]
impl Process for StoreConfig {
    async fn process(self, client: &Client) -> Result<()> {
        let StoreConfig {
            config_address,
            uri,
        } = self;

        store_config::process(client, config_address, uri).await
    }
}
