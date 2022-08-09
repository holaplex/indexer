//! Support features for the search indexer

mod client;

use std::fmt;

pub use client::{Args as ClientArgs, Client};
use indexer_core::{
    assets::AssetIdentifier,
    db::tables::{
        metadata_collection_keys, metadata_creators, metadata_jsons, metadatas,
        twitter_handle_name_services,
    },
    meilisearch::IndirectMetadataDocument,
};
use indexer_rabbitmq::search_indexer::{self, Message};
use reqwest::Url;

use crate::prelude::*;

/// Message identifier
#[derive(Debug, Clone, Copy)]
pub enum MessageId {
    /// The message was a direct document upsert
    Upsert,
    /// The message was an indirect upsert for a metadata account with the given
    /// mint
    IndirectMetadata(Pubkey),
}

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
                .map_err(|e| MessageError::new(e, MessageId::Upsert))?;

            Ok(())
        },
        Message::IndirectMetadata { index, mint } => {
            let mint_address = mint.to_string();
            let msg_id = MessageId::IndirectMetadata(mint);
            let doc = get_indirect_metadata(client, mint_address.clone())
                .await
                .map_err(|e| MessageError::new(e, msg_id))?;

            client
                .upsert_documents(
                    index,
                    Some(Document {
                        id: mint_address,
                        body: serde_json::to_value(doc)
                            .context("Failed to serialize metadata document")
                            .map_err(|e| MessageError::new(e, msg_id))?,
                    }),
                )
                .await
                .map_err(|e| MessageError::new(e, msg_id))?;

            Ok(())
        },
    }
}

async fn get_indirect_metadata(
    client: &Client,
    mint_address: String,
) -> Result<IndirectMetadataDocument> {
    let (
        metadata_address,
        name,
        image,
        collection_address,
        creator_address,
        creator_twitter_handle,
    ) = client
        .db()
        .run({
            let mint_address = mint_address.clone();

            move |conn| {
                metadatas::table
                    .inner_join(
                        metadata_jsons::table
                            .on(metadata_jsons::metadata_address.eq(metadatas::address)),
                    )
                    .left_join(
                        metadata_collection_keys::table
                            .on(metadatas::address.eq(metadata_collection_keys::metadata_address)),
                    )
                    .inner_join(
                        metadata_creators::table
                            .on(metadata_creators::metadata_address.eq(metadatas::address)),
                    )
                    .left_join(
                        twitter_handle_name_services::table.on(metadata_creators::creator_address
                            .eq(twitter_handle_name_services::wallet_address)),
                    )
                    .filter(metadatas::mint_address.eq(mint_address))
                    .filter(metadata_creators::verified.eq(true))
                    .filter(metadata_creators::position.eq(0))
                    .select((
                        metadatas::address,
                        metadatas::name,
                        metadata_jsons::image,
                        metadata_collection_keys::collection_address.nullable(),
                        metadata_creators::creator_address,
                        twitter_handle_name_services::twitter_handle.nullable(),
                    ))
                    .first::<(
                        String,
                        String,
                        Option<String>,
                        Option<String>,
                        String,
                        Option<String>,
                    )>(conn)
                    .context("Failed to load metadata JSON")
            }
        })
        .await?;

    let image = image
        .as_ref()
        .and_then(|i| Url::parse(i).ok())
        .and_then(|u| {
            let id = AssetIdentifier::new(&u);

            indexer_core::assets::proxy_url(client.proxy_args(), &id, Some(("width", "200")))
                .map(|o| o.map(|u| u.to_string()))
                .transpose()
        })
        .or_else(|| image.map(Ok))
        .transpose()?;

    Ok(IndirectMetadataDocument {
        metadata_address,
        mint_address,
        name,
        image,
        creator_address,
        creator_twitter_handle,
        collection_address,
    })
}
