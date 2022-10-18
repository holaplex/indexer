use indexer_core::db::{
    queries::{self},
    tables::{attributes, collection_mints, current_metadata_owners, metadata_jsons, metadatas},
};
use objects::attributes::AttributeGroup;
use serde_json::Value;
use services;

use super::{nft::Nft, prelude::*};

#[derive(Debug, Clone, GraphQLObject)]
pub struct CollectionDocument {
    pub id: String,
    pub name: String,
    pub image: String,
    pub magic_eden_id: Option<String>,
    pub verified_collection_address: Option<String>,
    pub twitter_url: Option<String>,
    pub discord_url: Option<String>,
    pub website_url: Option<String>,
}

impl From<serde_json::Value> for CollectionDocument {
    fn from(value: serde_json::Value) -> Self {
        Self {
            id: value
                .get("id")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            name: value
                .get("name")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            image: value
                .get("image")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            magic_eden_id: value
                .get("magic_eden_id")
                .and_then(Value::as_str)
                .map(Into::into),
            verified_collection_address: value
                .get("verified_collection_address")
                .and_then(Value::as_str)
                .map(Into::into),
            twitter_url: value
                .get("twitter_url")
                .and_then(Value::as_str)
                .map(Into::into),
            discord_url: value
                .get("discord_url")
                .and_then(Value::as_str)
                .map(Into::into),
            website_url: value
                .get("website_url")
                .and_then(Value::as_str)
                .map(Into::into),
        }
    }
}

pub type CollectionId = String;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionId,
    pub image: String,
    pub name: String,
    pub description: String,
    pub twitter_url: Option<String>,
    pub discord_url: Option<String>,
    pub website_url: Option<String>,
    pub magic_eden_id: Option<String>,
    pub verified_collection_address: Option<String>,
    pub pieces: i32,
    pub verified: bool,
    pub go_live_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'a> TryFrom<models::Collection<'a>> for Collection {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::Collection {
            id,
            image,
            name,
            description,
            twitter_url,
            discord_url,
            website_url,
            magic_eden_id,
            verified_collection_address,
            pieces,
            verified,
            go_live_at,
            created_at,
            updated_at,
        }: models::Collection,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.to_string(),
            image: image.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            twitter_url: twitter_url.map(Into::into),
            discord_url: discord_url.map(Into::into),
            website_url: website_url.map(Into::into),
            magic_eden_id: magic_eden_id.map(Into::into),
            verified_collection_address: verified_collection_address.map(Into::into),
            pieces: pieces.try_into().unwrap_or_default(),
            verified,
            go_live_at: DateTime::from_utc(go_live_at, Utc),
            created_at: DateTime::from_utc(created_at, Utc),
            updated_at: DateTime::from_utc(updated_at, Utc),
        })
    }
}

#[graphql_object(Context = AppContext)]
impl Collection {
    pub fn id(&self) -> &CollectionId {
        &self.id
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn twitter_url(&self) -> Option<&str> {
        self.twitter_url.as_deref()
    }

    pub fn discord_url(&self) -> Option<&str> {
        self.discord_url.as_deref()
    }

    pub fn website_url(&self) -> Option<&str> {
        self.website_url.as_deref()
    }

    pub fn magic_eden_id(&self) -> Option<&str> {
        self.magic_eden_id.as_deref()
    }

    pub fn verified_collection_address(&self) -> Option<&str> {
        self.verified_collection_address.as_deref()
    }

    pub fn pieces(&self) -> i32 {
        self.pieces
    }

    pub fn verified(&self) -> bool {
        self.verified
    }

    pub fn go_live_at(&self) -> DateTime<Utc> {
        self.go_live_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // nfts data loader

    pub async fn nfts(&self, ctx: &AppContext, limit: i32, offset: i32) -> FieldResult<Vec<Nft>> {
        let conn = ctx.shared.db.get()?;

        let nfts: Vec<models::Nft> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadata_jsons::metadata_address.eq(metadatas::address)),
            )
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .inner_join(
                collection_mints::table.on(metadatas::mint_address.eq(collection_mints::mint)),
            )
            .filter(collection_mints::collection_id.eq(self.id.clone()))
            .select(queries::metadatas::NFT_COLUMNS)
            .offset(offset.into())
            .limit(limit.into())
            .load(&conn)
            .context("Failed to load NFTs")?;

        nfts.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.shared.db.get()?;

        let metadata_attributes: Vec<models::MetadataAttribute> = attributes::table
            .inner_join(metadatas::table.on(attributes::metadata_address.eq(metadatas::address)))
            .inner_join(
                collection_mints::table.on(metadatas::mint_address.eq(collection_mints::mint)),
            )
            .filter(collection_mints::collection_id.eq(&self.id))
            .select(attributes::all_columns)
            .load(&conn)
            .context("Failed to load metadata attributes")?;

        services::attributes::group(metadata_attributes)
    }

    #[graphql(description = "Count of NFTs in the collection.")]
    async fn nft_count(&self, context: &AppContext) -> FieldResult<Option<scalars::I64>> {
        Ok(context
            .mr_collection_nft_count_loader
            .load(self.id.clone())
            .await?
            .map(|dataloaders::collection::CollectionNftCount(nft_count)| nft_count))
    }

    #[graphql(
        description = "Count of wallets that currently hold at least one NFT from the collection."
    )]
    pub async fn holder_count(&self, ctx: &AppContext) -> FieldResult<Option<scalars::I64>> {
        Ok(ctx
            .mr_collection_holders_count_loader
            .load(self.id.clone())
            .await?
            .map(|dataloaders::collection::CollectionHoldersCount(nft_count)| nft_count))
    }

    pub async fn activities(
        &self,
        ctx: &AppContext,
        event_types: Option<Vec<String>>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<objects::nft::NftActivity>> {
        let conn = ctx.shared.db.get()?;
        let rows = queries::collections::mr_collection_activities(
            &conn,
            &self.id,
            event_types,
            limit,
            offset,
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}
