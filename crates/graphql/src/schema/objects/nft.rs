use async_trait::async_trait;
use indexer_core::{
    assets::{proxy_url, AssetIdentifier, ImageSize},
    db::{
        queries,
        tables::{bid_receipts, collection_stats, listing_receipts, metadata_jsons},
    },
    util::unix_timestamp,
    uuid::Uuid,
};
use juniper::graphql_interface;
use objects::{
    ah_listing::AhListing, ah_offer::Offer, ah_purchase::Purchase, auction_house::AuctionHouse,
    profile::TwitterProfile, wallet::Wallet,
};
use reqwest::Url;
use scalars::{PublicKey, U64};
use serde_json::Value;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct NftAttribute {
    pub metadata_address: String,
    pub value: Option<String>,
    pub trait_type: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl NftAttribute {
    pub fn metadata_address(&self) -> &str {
        &self.metadata_address
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn trait_type(&self) -> Option<&str> {
        self.trait_type.as_deref()
    }
}

impl<'a> TryFrom<models::MetadataAttribute<'a>> for NftAttribute {
    type Error = Error;

    fn try_from(
        models::MetadataAttribute {
            metadata_address,
            value,
            trait_type,
            ..
        }: models::MetadataAttribute,
    ) -> Result<Self> {
        Ok(Self {
            metadata_address: metadata_address.into_owned(),
            value: value.map(Cow::into_owned),
            trait_type: trait_type.map(Cow::into_owned),
        })
    }
}

#[derive(Debug, Clone)]
/// An NFT file
pub struct NftFile {
    pub metadata_address: String,
    pub uri: String,
    pub file_type: String,
}

#[graphql_object(Context = AppContext)]
impl NftFile {
    pub fn metadata_address(&self) -> &str {
        &self.metadata_address
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn file_type(&self) -> &str {
        &self.file_type
    }
}

impl<'a> From<models::MetadataFile<'a>> for NftFile {
    fn from(
        models::MetadataFile {
            metadata_address,
            uri,
            file_type,
            ..
        }: models::MetadataFile,
    ) -> Self {
        Self {
            metadata_address: metadata_address.into_owned(),
            uri: uri.into_owned(),
            file_type: file_type.into_owned(),
        }
    }
}

#[derive(Debug, Clone)]
/// An NFT creator
pub struct NftCreator {
    pub address: String,
    pub metadata_address: String,
    pub share: i32,
    pub verified: bool,
    pub position: Option<i32>,
    pub twitter_handle: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl NftCreator {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn metadata_address(&self) -> &str {
        &self.metadata_address
    }

    pub fn share(&self) -> i32 {
        self.share
    }

    pub fn verified(&self) -> bool {
        self.verified
    }

    pub fn position(&self) -> Option<i32> {
        self.position
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<(Option<String>, models::MetadataCreator<'a>)> for NftCreator {
    fn from(
        (
            twitter_handle,
            models::MetadataCreator {
                creator_address,
                metadata_address,
                share,
                verified,
                position,
            },
        ): (Option<String>, models::MetadataCreator),
    ) -> Self {
        Self {
            address: creator_address.into_owned(),
            metadata_address: metadata_address.into_owned(),
            share,
            verified,
            position,
            twitter_handle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NftOwner {
    pub address: String,
    pub associated_token_account_address: String,
    pub twitter_handle: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl NftOwner {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn associated_token_account_address(&self) -> &str {
        &self.associated_token_account_address
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct NftActivity {
    pub id: Uuid,
    pub metadata: PublicKey<BaseNft>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub marketplace_program_address: String,
    pub price: U64,
    pub created_at: DateTime<Utc>,
    pub wallets: Vec<Wallet>,
    pub activity_type: String,
}

impl TryFrom<models::NftActivity> for NftActivity {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::NftActivity {
            id,
            metadata,
            auction_house,
            marketplace_program,
            price,
            created_at,
            wallets,
            wallet_twitter_handles,
            activity_type,
        }: models::NftActivity,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id,
            metadata: metadata.into(),
            auction_house: auction_house.into(),
            marketplace_program_address: marketplace_program,
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
            wallets: wallets
                .into_iter()
                .zip(wallet_twitter_handles.into_iter())
                .map(|(address, twitter_handle)| Wallet::new(address.into(), twitter_handle))
                .collect(),
            activity_type,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl NftActivity {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn metadata(&self) -> &PublicKey<BaseNft> {
        &self.metadata
    }

    fn price(&self) -> U64 {
        self.price
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallets(&self) -> &Vec<Wallet> {
        &self.wallets
    }

    fn activity_type(&self) -> &str {
        &self.activity_type
    }

    fn marketplace_program_address(&self) -> &str {
        &self.marketplace_program_address
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<BaseNft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .store_auction_houses_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
}

#[graphql_interface(for = [BaseNft, CollectionNft])]
#[async_trait]
pub trait Nft {
    fn address(&self) -> &str;
    fn name(&self) -> &str;
    fn seller_fee_basis_points(&self) -> i32;
    fn mint_address(&self) -> &str;
    fn token_account_address(&self) -> &str;
    fn primary_sale_happened(&self) -> bool;
    fn update_authority_address(&self) -> &str;
    fn uri(&self) -> &str;
    fn description(&self) -> &str;
    fn animation_url(&self) -> Option<&str>;
    fn external_url(&self) -> Option<&str>;
    fn category(&self) -> &str;
    fn model(&self) -> Option<String>;
    fn slot(&self) -> Option<i32>;

    fn image(
        &self,
        #[graphql(desc = r"Image width possible values are:
    - 0 (Original size)
    - 100 (Tiny)
    - 400 (XSmall)
    - 600 (Small)
    - 800 (Medium)
    - 1400 (Large)

    Any other value will return the original image size.

    If no value is provided, it will return XSmall")]
        width: Option<i32>,
        ctx: &AppContext,
    ) -> FieldResult<String>;
    async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>>;
    async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>>;
    async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>>;
    async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>>;
    async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>>;
    async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>>;
    async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>>;
    async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>>;
    async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<CollectionNft>>;
    async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>>;

    fn parser(&self) -> Option<&str>;
}

/// An NFT
#[derive(Debug, Clone, GraphQLObject)]
#[graphql(impl = NftValue, Context = AppContext)]
pub struct BaseNft {
    pub address: String,
    pub name: String,
    pub seller_fee_basis_points: i32,
    pub mint_address: String,
    pub token_account_address: String,
    pub primary_sale_happened: bool,
    pub update_authority_address: String,
    pub uri: String,
    pub description: String,
    pub image: String,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub category: String,
    pub model: Option<String>,
    pub slot: Option<i32>,
}

impl TryFrom<models::Nft> for BaseNft {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::Nft {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            token_account_address,
            primary_sale_happened,
            update_authority_address,
            uri,
            description,
            image,
            animation_url,
            external_url,
            category,
            model,
            slot,
        }: models::Nft,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            token_account_address,
            primary_sale_happened,
            update_authority_address,
            uri,
            description: description.unwrap_or_else(String::new),
            image: image.unwrap_or_else(String::new),
            animation_url,
            external_url,
            category: category.unwrap_or_else(String::new),
            model,
            slot: slot.map(TryInto::try_into).transpose()?,
        })
    }
}

#[async_trait]
impl Nft for BaseNft {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn slot(&self) -> Option<i32> {
        self.slot
    }

    fn model(&self) -> Option<String> {
        self.model.as_ref().map(ToString::to_string)
    }

    fn address(&self) -> &str {
        &self.address
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    fn mint_address(&self) -> &str {
        &self.mint_address
    }

    fn token_account_address(&self) -> &str {
        &self.token_account_address
    }

    fn primary_sale_happened(&self) -> bool {
        self.primary_sale_happened
    }

    fn update_authority_address(&self) -> &str {
        &self.update_authority_address
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn animation_url(&self) -> Option<&str> {
        self.animation_url.as_deref()
    }

    fn external_url(&self) -> Option<&str> {
        self.external_url.as_deref()
    }

    /// The JSON parser with which the NFT was processed by the indexer
    ///
    /// - `"full"` indicates the full Metaplex standard-compliant parser was
    ///   used.
    /// - `"minimal"` (provided with an optional description of an error)
    ///   indicates the full model failed to parse and a more lenient fallback
    ///   parser with fewer fields was used instead.
    fn parser(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
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

    async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        ctx.nft_creators_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        ctx.nft_attributes_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        ctx.nft_owner_loader
            .load(self.mint_address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>> {
        ctx.nft_activities_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>> {
        ctx.ah_listings_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>> {
        ctx.purchases_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>> {
        ctx.offers_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>> {
        ctx.nft_files_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<CollectionNft>> {
        ctx.nft_collection_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>> {
        if let Some(slot) = self.slot {
            let shared = ctx.shared.clone();

            tokio::task::spawn_blocking(move || {
                shared
                    .rpc
                    .get_block_time(slot.try_into().unwrap_or_default())
                    .context("RPC call for block time failed")
                    .and_then(|s| unix_timestamp(s).map(|t| DateTime::<Utc>::from_utc(t, Utc)))
            })
            .await
            .expect("Blocking task panicked")
            .map(Some)
            .map_err(Into::into)
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionNft(BaseNft);

#[graphql_object(impl = NftValue, Context = AppContext)]
impl CollectionNft {
    fn nft_count(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = collection_stats::table
            .filter(collection_stats::collection_address.eq(&self.0.mint_address))
            .select(collection_stats::nft_count)
            .get_result::<i64>(&conn)
            .context("failed to load NFT count for this collection")?;

        Ok(count.try_into()?)
    }

    fn floor_price(&self, context: &AppContext) -> FieldResult<Option<i32>> {
        let conn = context.shared.db.get()?;

        let count = collection_stats::table
            .filter(collection_stats::collection_address.eq(&self.0.mint_address))
            .select(collection_stats::floor_price)
            .get_result::<Option<i64>>(&conn)
            .context("failed to load floor price for this collection")?;

        match count {
            Some(price) => {
                let downcast = price.try_into()?;
                Ok(Some(downcast))
            },
            None => Ok(None),
        }
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn uri(&self) -> &str {
        self.0.uri()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn slot(&self) -> Option<i32> {
        self.0.slot()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn model(&self) -> Option<String> {
        self.0.model()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn address(&self) -> &str {
        self.0.address()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn name(&self) -> &str {
        self.0.name()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn seller_fee_basis_points(&self) -> i32 {
        self.0.seller_fee_basis_points()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn mint_address(&self) -> &str {
        self.0.mint_address()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn token_account_address(&self) -> &str {
        self.0.token_account_address()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn primary_sale_happened(&self) -> bool {
        self.0.primary_sale_happened()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn update_authority_address(&self) -> &str {
        self.0.update_authority_address()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn description(&self) -> &str {
        self.0.description()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn category(&self) -> &str {
        self.0.category()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn animation_url(&self) -> Option<&str> {
        self.0.animation_url()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn external_url(&self) -> Option<&str> {
        self.0.external_url()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn parser(&self) -> Option<&str> {
        self.0.parser()
    }

    #[graphql(deprecated = "use `...on Nft`")]
    fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
        self.0.image(width, ctx)
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        self.0.creators(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        self.0.attributes(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        self.0.owner(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>> {
        self.0.activities(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>> {
        self.0.listings(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>> {
        self.0.purchases(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>> {
        self.0.offers(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>> {
        self.0.files(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<CollectionNft>> {
        self.0.collection(ctx).await
    }

    #[graphql(deprecated = "use `...on Nft`")]
    async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>> {
        self.0.created_at(ctx).await
    }
}

#[async_trait]
impl Nft for CollectionNft {
    fn uri(&self) -> &str {
        self.0.uri()
    }

    fn slot(&self) -> Option<i32> {
        self.0.slot()
    }

    fn model(&self) -> Option<String> {
        self.0.model()
    }

    fn address(&self) -> &str {
        self.0.address()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn seller_fee_basis_points(&self) -> i32 {
        self.0.seller_fee_basis_points()
    }

    fn mint_address(&self) -> &str {
        self.0.mint_address()
    }

    fn token_account_address(&self) -> &str {
        self.0.token_account_address()
    }

    fn primary_sale_happened(&self) -> bool {
        self.0.primary_sale_happened()
    }

    fn update_authority_address(&self) -> &str {
        self.0.update_authority_address()
    }

    fn description(&self) -> &str {
        self.0.description()
    }

    fn category(&self) -> &str {
        self.0.category()
    }

    fn animation_url(&self) -> Option<&str> {
        self.0.animation_url()
    }

    fn external_url(&self) -> Option<&str> {
        self.0.external_url()
    }

    /// The JSON parser with which the NFT was processed by the indexer
    ///
    /// - `"full"` indicates the full Metaplex standard-compliant parser was
    ///   used.
    /// - `"minimal"` (provided with an optional description of an error)
    ///   indicates the full model failed to parse and a more lenient fallback
    ///   parser with fewer fields was used instead.
    fn parser(&self) -> Option<&str> {
        self.0.parser()
    }

    fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
        self.0.image(width, ctx)
    }

    async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        self.0.creators(ctx).await
    }

    async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        self.0.attributes(ctx).await
    }

    async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        self.0.owner(ctx).await
    }

    async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>> {
        self.0.activities(ctx).await
    }

    async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>> {
        self.0.listings(ctx).await
    }

    async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>> {
        self.0.purchases(ctx).await
    }

    async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>> {
        self.0.offers(ctx).await
    }

    async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>> {
        self.0.files(ctx).await
    }

    async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<CollectionNft>> {
        self.0.collection(ctx).await
    }

    async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>> {
        self.0.created_at(ctx).await
    }
}

impl TryFrom<models::Nft> for CollectionNft {
    type Error = <BaseNft as TryFrom<models::Nft>>::Error;

    fn try_from(value: models::Nft) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

#[derive(Debug, Clone)]
pub struct NftCount {
    creators: Vec<PublicKey<NftCreator>>,
}

impl NftCount {
    #[must_use]
    pub fn new(creators: Vec<PublicKey<NftCreator>>) -> Self {
        Self { creators }
    }
}

#[graphql_object(Context = AppContext)]
impl NftCount {
    fn total(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::total(&conn, &self.creators)?;

        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "a list of auction house public keys")))]
    fn listed(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::listed(&conn, &self.creators, auction_houses.as_deref())?;

        Ok(count.try_into()?)
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct MetadataJson {
    pub address: String,
    pub name: String,
    pub mint_address: String,
    pub image: Option<String>,
    pub creator_address: Option<String>,
    pub creator_twitter_handle: Option<String>,
}

impl From<serde_json::Value> for MetadataJson {
    fn from(value: serde_json::Value) -> Self {
        Self {
            address: value
                .get("id")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            name: value
                .get("name")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            mint_address: value
                .get("mint_address")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            image: value.get("image").and_then(Value::as_str).map(Into::into),
            creator_address: value
                .get("creator_address")
                .and_then(Value::as_str)
                .map(Into::into),
            creator_twitter_handle: value
                .get("creator_twitter_handle")
                .and_then(Value::as_str)
                .map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NftsStats;

#[graphql_object(Context = AppContext)]
impl NftsStats {
    #[graphql(description = "The total number of indexed NFTs")]
    fn total_nfts(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count: i64 = metadata_jsons::table
            .count()
            .get_result(&conn)
            .context("failed to load total NFTs count")?;

        Ok(count.try_into()?)
    }

    #[graphql(description = "The total number of buy-now listings")]
    fn buy_now_listings(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count: i64 = listing_receipts::table
            .filter(listing_receipts::price.is_not_null())
            .filter(listing_receipts::purchase_receipt.is_null())
            .filter(listing_receipts::canceled_at.is_null())
            .count()
            .get_result(&conn)
            .context("failed to load listed nfts count")?;

        Ok(count.try_into()?)
    }

    #[graphql(description = "The total number of NFTs with active offers")]
    fn nfts_with_active_offers(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count: i64 = bid_receipts::table
            .filter(bid_receipts::purchase_receipt.is_null())
            .filter(bid_receipts::canceled_at.is_null())
            .count()
            .get_result(&conn)
            .context("failed to load listed nfts count")?;

        Ok(count.try_into()?)
    }
}
