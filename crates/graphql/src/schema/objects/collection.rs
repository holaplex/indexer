use dolphin_stats::{market_stats_endpoint, MarketStats, MarketStatsResponse};
use indexer_core::{
    assets::{proxy_url, AssetIdentifier, ImageSize},
    db::{
        queries::{self, metadatas::CollectionNftOptions},
        tables::attribute_groups,
    },
    pubkeys,
};
use objects::attributes::AttributeGroup;
use reqwest::Url;
use serde_json::Value;
use services;

use super::{nft::Nft, prelude::*};
use crate::schema::{
    enums::{NftSort, OrderDirection},
    query_root::AttributeFilter,
    scalars::{Numeric, I64, U64},
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

    pub async fn trends(&self, context: &AppContext) -> FieldResult<Option<CollectionTrend>> {
        context
            .mr_collection_trends_loader
            .load(self.id.clone())
            .await
            .map_err(Into::into)
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

        let nfts = queries::metadatas::mr_collection_nfts(
            &conn,
            CollectionNftOptions {
                collection: self.id.clone(),
                auction_house,
                attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
                marketplace_program,
                sort_by: sort_by.map(Into::into),
                order: order.map(Into::into),
                limit: limit.try_into()?,
                offset: offset.try_into()?,
            },
            pubkeys::OPENSEA_AUCTION_HOUSE.to_string(),
        )?;

        nfts.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.shared.db.get()?;

        let variants: Vec<models::AttributeGroup> = attribute_groups::table
            .filter(attribute_groups::collection_id.eq(self.id.clone()))
            .select(attribute_groups::all_columns)
            .load(&conn)
            .context("failed to get attribute groups")?;

        services::attributes::group(variants)
    }

    #[graphql(
        description = "Count of wallets that currently hold at least one NFT from the collection."
    )]
    pub async fn holder_count(&self, ctx: &AppContext) -> FieldResult<Option<I64>> {
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

    pub async fn timeseries(
        &self,
        ctx: &AppContext,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> FieldResult<Timeseries> {
        let http = &ctx.shared.http;
        let dolphin_key = &ctx.shared.dolphin_key;
        let url = market_stats_endpoint(&self.id, &start_time, &end_time)?;

        let json = http
            .get(url.clone())
            .header("Authorization", dolphin_key)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<MarketStatsResponse>()
            .await?
            .into_inner(|| &url)?;

        let MarketStats {
            floor_data,
            listed_data,
            holder_data,
            ..
        } = json;

        let floor_price = floor_data.iter().map(Into::into).collect();
        let listed_count = listed_data.iter().map(Into::into).collect();
        let holder_count = holder_data.iter().map(Into::into).collect();

        Ok(Timeseries {
            floor_price,
            listed_count,
            holder_count,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct Datapoint {
    pub timestamp: DateTime<Utc>,
    pub value: U64,
}

impl From<&(u64, serde_json::Number)> for Datapoint {
    fn from(&(ts, ref num): &(u64, serde_json::Number)) -> Self {
        Self {
            timestamp: dolphin_stats::get_datapoint_timestamp(ts).unwrap_or_default(),
            value: num.as_u64().unwrap_or_default().into(),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct Timeseries {
    pub floor_price: Vec<Datapoint>,
    pub listed_count: Vec<Datapoint>,
    pub holder_count: Vec<Datapoint>,
}

// TODO: use collection identifier for the data loader instead of string
#[derive(Debug, Clone)]
pub struct CollectionIdentifier(pub String);

#[derive(Debug, Clone)]
pub struct CollectionTrend {
    pub collection: String,
    pub floor_1d: Numeric,
    pub floor_7d: Numeric,
    pub floor_30d: Numeric,
    pub volume_1d: Numeric,
    pub volume_7d: Numeric,
    pub volume_30d: Numeric,
    pub listed_1d: I64,
    pub listed_7d: I64,
    pub listed_30d: I64,
    pub last_volume_1d: Numeric,
    pub last_volume_7d: Numeric,
    pub last_volume_30d: Numeric,
    pub last_listed_1d: I64,
    pub last_listed_7d: I64,
    pub last_listed_30d: I64,
    pub last_floor_1d: Numeric,
    pub last_floor_7d: Numeric,
    pub last_floor_30d: Numeric,
    pub change_volume_1d: Option<i32>,
    pub change_volume_7d: Option<i32>,
    pub change_volume_30d: Option<i32>,
    pub change_floor_1d: Option<i32>,
    pub change_floor_7d: Option<i32>,
    pub change_floor_30d: Option<i32>,
    pub change_listed_1d: Option<i32>,
    pub change_listed_7d: Option<i32>,
    pub change_listed_30d: Option<i32>,
}

impl<'a> TryFrom<models::DolphinStats<'a>> for CollectionTrend {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::DolphinStats {
            collection_symbol,
            floor_1d,
            floor_7d,
            floor_30d,
            last_floor_1d,
            last_floor_7d,
            last_floor_30d,
            change_floor_1d,
            change_floor_7d,
            change_floor_30d,
            volume_1d,
            volume_7d,
            volume_30d,
            last_volume_1d,
            last_volume_7d,
            last_volume_30d,
            change_volume_1d,
            change_volume_7d,
            change_volume_30d,
            listed_1d,
            listed_7d,
            listed_30d,
            last_listed_1d,
            last_listed_7d,
            last_listed_30d,
            change_listed_1d,
            change_listed_7d,
            change_listed_30d,
        }: models::DolphinStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            collection: collection_symbol.into_owned(),
            floor_1d: floor_1d.into(),
            floor_7d: floor_7d.into(),
            floor_30d: floor_30d.into(),
            volume_1d: volume_1d.into(),
            volume_7d: volume_7d.into(),
            volume_30d: volume_30d.into(),
            listed_1d: listed_1d.into(),
            listed_7d: listed_7d.into(),
            listed_30d: listed_30d.into(),
            last_volume_1d: last_volume_1d.into(),
            last_volume_7d: last_volume_7d.into(),
            last_volume_30d: last_volume_30d.into(),
            last_listed_1d: last_listed_1d.into(),
            last_listed_7d: last_listed_7d.into(),
            last_listed_30d: last_listed_30d.into(),
            last_floor_1d: last_floor_1d.into(),
            last_floor_7d: last_floor_7d.into(),
            last_floor_30d: last_floor_30d.into(),
            change_volume_1d: change_volume_1d.map(Into::into),
            change_volume_7d: change_volume_7d.map(Into::into),
            change_volume_30d: change_volume_30d.map(Into::into),
            change_floor_1d: change_floor_1d.map(Into::into),
            change_floor_7d: change_floor_7d.map(Into::into),
            change_floor_30d: change_floor_30d.map(Into::into),
            change_listed_1d: change_listed_1d.map(Into::into),
            change_listed_7d: change_listed_7d.map(Into::into),
            change_listed_30d: change_listed_30d.map(Into::into),
        })
    }
}

#[graphql_object(Context = AppContext)]
impl CollectionTrend {
    pub fn floor_1d(&self) -> &Numeric {
        &self.floor_1d
    }

    pub fn floor_7d(&self) -> &Numeric {
        &self.floor_7d
    }

    pub fn floor_30d(&self) -> &Numeric {
        &self.floor_30d
    }

    pub fn volume_1d(&self) -> &Numeric {
        &self.volume_1d
    }

    pub fn volume_7d(&self) -> &Numeric {
        &self.volume_7d
    }

    pub fn volume_30d(&self) -> &Numeric {
        &self.volume_30d
    }

    pub fn listed_1d(&self) -> &I64 {
        &self.listed_1d
    }

    pub fn listed_7d(&self) -> &I64 {
        &self.listed_7d
    }

    pub fn listed_30d(&self) -> &I64 {
        &self.listed_30d
    }

    pub fn last_listed_1d(&self) -> &I64 {
        &self.last_listed_1d
    }

    pub fn last_listed_7d(&self) -> &I64 {
        &self.last_listed_7d
    }

    pub fn last_listed_30d(&self) -> &I64 {
        &self.last_listed_30d
    }

    pub fn last_volume_1d(&self) -> &Numeric {
        &self.last_volume_1d
    }

    pub fn last_volume_7d(&self) -> &Numeric {
        &self.last_volume_7d
    }

    pub fn last_volume_30d(&self) -> &Numeric {
        &self.last_volume_30d
    }

    pub fn last_floor_1d(&self) -> &Numeric {
        &self.last_floor_1d
    }

    pub fn last_floor_7d(&self) -> &Numeric {
        &self.last_floor_7d
    }

    pub fn last_floor_30d(&self) -> &Numeric {
        &self.last_floor_30d
    }

    pub fn change_floor_1d(&self) -> Option<i32> {
        self.change_floor_1d
    }

    pub fn change_floor_7d(&self) -> Option<i32> {
        self.change_floor_7d
    }

    pub fn change_floor_30d(&self) -> Option<i32> {
        self.change_floor_30d
    }

    pub fn change_volume_1d(&self) -> Option<i32> {
        self.change_volume_1d
    }

    pub fn change_volume_7d(&self) -> Option<i32> {
        self.change_volume_7d
    }

    pub fn change_volume_30d(&self) -> Option<i32> {
        self.change_volume_30d
    }

    pub fn change_listed_1d(&self) -> Option<i32> {
        self.change_listed_1d
    }

    pub fn change_listed_7d(&self) -> Option<i32> {
        self.change_listed_7d
    }

    pub fn change_listed_30d(&self) -> Option<i32> {
        self.change_listed_30d
    }

    pub async fn collection(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<objects::collection::Collection>> {
        ctx.generic_collection_loader
            .load(self.collection.clone())
            .await
            .map_err(Into::into)
    }
}
