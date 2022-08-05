//! Support features for the HTTP indexer

pub(self) mod client;
mod metadata_json;
mod store_config;

use std::fmt;

pub use client::{Args as ClientArgs, Client};
use indexer_rabbitmq::http_indexer::{Entity, MetadataJson, StoreConfig};

use crate::prelude::*;

/// Message identifier
#[derive(Debug, Clone, Copy)]
pub enum MessageId {
    /// A JSON document associated with the metadata account with the given key
    MetadataJson(Pubkey),
    /// A store config JSON document associated with the given key
    StoreConfig(Pubkey),
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MetadataJson(k) => write!(f, "JSON for metadata account {}", k),
            Self::StoreConfig(k) => write!(f, "store config at {}", k),
        }
    }
}

/// Define processing logic for an incoming entity type
#[async_trait::async_trait]
pub trait Process: Entity {
    /// Process and consume an incoming entity
    async fn process(self, client: &Client) -> MessageResult<MessageId>;
}

#[async_trait::async_trait]
impl Process for MetadataJson {
    async fn process(self, client: &Client) -> MessageResult<MessageId> {
        let MetadataJson {
            meta_address,
            first_verified_creator,
            uri,
            slot_info,
        } = self;

        metadata_json::process(client, meta_address, first_verified_creator, uri, slot_info)
            .await
            .map_err(|e| MessageError::new(e, MessageId::MetadataJson(meta_address)))
    }
}

#[async_trait::async_trait]
impl Process for StoreConfig {
    async fn process(self, client: &Client) -> MessageResult<MessageId> {
        let StoreConfig {
            config_address,
            uri,
        } = self;

        store_config::process(client, config_address, uri)
            .await
            .map_err(|e| MessageError::new(e, MessageId::StoreConfig(config_address)))
    }
}
