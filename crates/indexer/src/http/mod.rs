//! Support features for the HTTP indexer

pub(self) mod client;
mod metadata_json;
mod store_config_json;

pub use client::Client;
use indexer_rabbitmq::http_indexer::{Entity, MetadataJson, StoreConfig};

use crate::prelude::*;

#[async_trait::async_trait]
pub trait Process: Entity {
    async fn process(self) -> Result<()>;
}

#[async_trait::async_trait]
impl Process for MetadataJson {
    async fn process(self) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl Process for StoreConfig {
    async fn process(self) -> Result<()> {
        Ok(())
    }
}
