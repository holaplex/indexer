use indexer_core::{
    assets::{proxy_url, AssetIdentifier, ImageSize},
    db::{
        queries::{self, metadatas::CollectionNftOptions},
        tables::{attributes, collection_mints, metadatas},
    },
};
use objects::attributes::AttributeGroup;
use reqwest::Url;
use serde_json::Value;
use services;

use super::{nft::Nft, prelude::*};
use crate::schema::{
    enums::{NftSort, OrderDirection},
    query_root::AttributeFilter,
};

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

    #[graphql(description = r"Get the original URL of the image as stored in the NFT's metadata")]
    pub fn image_original(&self) -> &str {
        &self.image
    }

    pub fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
        let url = Url::parse(&self.image);
        let id = if let Ok(ref url) = url {
            AssetIdentifier::new(url)
        } else {
            return Ok(self.image.clone());
        };

        let width = ImageSize::from(width.unwrap_or(ImageSize::XSmall as i32));
        let width_str = (width as i32).to_string();

        Ok(
            proxy_url(&ctx.shared.asset_proxy, &id, Some(("width", &*width_str)))?
                .map_or_else(|| self.image.clone(), |u| u.to_string()),
        )
    }

    pub async fn nfts(
        &self,
        ctx: &AppContext,
        limit: i32,
        offset: i32,
        sort_by: Option<NftSort>,
        order: Option<OrderDirection>,
        marketplace_program: Option<String>,
        auction_house: Option<String>,
        attributes: Option<Vec<AttributeFilter>>,
    ) -> FieldResult<Vec<Nft>> {
        let conn = ctx.shared.db.get()?;

        let nfts = queries::metadatas::mr_collection_nfts(&conn, CollectionNftOptions {
            collection: self.id.clone(),
            auction_house,
            attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
            marketplace_program,
            sort_by: sort_by.map(Into::into),
            order: order.map(Into::into),
            limit: limit.try_into()?,
            offset: offset.try_into()?,
        })?;

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
