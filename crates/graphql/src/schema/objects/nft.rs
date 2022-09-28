use indexer_core::{
    assets::{proxy_url, AssetIdentifier, ImageSize},
    bigdecimal::ToPrimitive,
    db::{
        queries, sql_query,
        sql_types::Text,
        tables::{
            attributes, auction_houses, bid_receipts, listing_receipts, listings,
            metadata_collection_keys, metadata_jsons, metadatas,
        },
    },
    pubkeys,
    url::Url,
    util::unix_timestamp,
    uuid::Uuid,
};
use objects::{
    ah_listing::AhListing, ah_offer::Offer, ah_purchase::Purchase, attributes::AttributeGroup,
    auction_house::AuctionHouse, profile::TwitterProfile, wallet::Wallet,
};
use scalars::{PublicKey, U64};
use serde_json::Value;
use services;

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
    pub metadata: PublicKey<Nft>,
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

    fn metadata(&self) -> &PublicKey<Nft> {
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

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .auction_house_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
/// An NFT
pub struct Nft {
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

impl TryFrom<models::Nft> for Nft {
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

#[graphql_object(Context = AppContext)]
impl Nft {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn mint_address(&self) -> &str {
        &self.mint_address
    }

    pub fn token_account_address(&self) -> &str {
        &self.token_account_address
    }

    pub fn primary_sale_happened(&self) -> bool {
        self.primary_sale_happened
    }

    pub fn update_authority_address(&self) -> &str {
        &self.update_authority_address
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    /// The JSON parser with which the NFT was processed by the indexer
    ///
    /// - `"full"` indicates the full Metaplex standard-compliant parser was
    ///   used.
    /// - `"minimal"` (provided with an optional description of an error)
    ///   indicates the full model failed to parse and a more lenient fallback
    ///   parser with fewer fields was used instead.
    pub fn parser(&self) -> Option<&str> {
        self.model.as_deref()
    }

    #[graphql(arguments(width(description = r"Image width possible values are:
- 0 (Original size)
- 100 (Tiny)
- 400 (XSmall)
- 600 (Small)
- 800 (Medium)
- 1400 (Large)

Any other value will return the original image size.

If no value is provided, it will return XSmall")))]
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

    #[graphql(description = r"Get the original URL of the image as stored in the NFT's metadata")]
    pub fn image_original(&self) -> &str {
        &self.image
    }

    pub fn animation_url(&self) -> Option<&str> {
        self.animation_url.as_deref()
    }

    pub fn external_url(&self) -> Option<&str> {
        self.external_url.as_deref()
    }

    pub async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        ctx.nft_creators_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        ctx.nft_attributes_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        ctx.nft_owner_loader
            .load(self.mint_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>> {
        ctx.nft_activities_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>> {
        ctx.ah_listings_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>> {
        ctx.purchases_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>> {
        ctx.offers_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>> {
        ctx.nft_files_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<Collection>> {
        ctx.nft_collection_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>> {
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
pub struct Collection(pub Nft);

impl From<Nft> for Collection {
    fn from(nft: Nft) -> Self {
        Self(nft)
    }
}

#[graphql_object(Context = AppContext)]
impl Collection {
    fn nft(&self) -> &Nft {
        &self.0
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.shared.db.get()?;

        let metadata_attributes: Vec<models::MetadataAttribute> =
            attributes::table
                .inner_join(metadata_collection_keys::table.on(
                    attributes::metadata_address.eq(metadata_collection_keys::metadata_address),
                ))
                .filter(metadata_collection_keys::collection_address.eq(&self.0.mint_address))
                .filter(metadata_collection_keys::verified.eq(true))
                .select(attributes::all_columns)
                .load(&conn)
                .context("Failed to load metadata attributes")?;

        services::attributes::group(metadata_attributes)
    }

    pub async fn activities(
        &self,
        ctx: &AppContext,
        event_types: Option<Vec<String>>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<NftActivity>> {
        let conn = ctx.shared.db.get()?;
        let rows = queries::collections::collection_activities(
            &conn,
            &self.0.mint_address,
            event_types,
            limit,
            offset,
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(description = "Lowest price of currently listed NFTs in the collection.")]
    async fn floor_price(&self, context: &AppContext) -> FieldResult<Option<scalars::I64>> {
        Ok(context
            .collection_floor_price_loader
            .load(self.0.mint_address.clone().into())
            .await?
            .map(|dataloaders::collection::CollectionFloorPrice(floor_price)| floor_price))
    }

    #[graphql(
        description = "Count of wallets that currently hold at least one NFT from the collection."
    )]
    pub async fn holder_count(&self, ctx: &AppContext) -> FieldResult<Option<scalars::I64>> {
        Ok(ctx
            .collection_holders_count_loader
            .load(self.0.mint_address.clone().into())
            .await?
            .map(|dataloaders::collection::CollectionHoldersCount(nft_count)| nft_count))
    }

    #[graphql(description = "Count of active listings of NFTs in the collection.")]
    pub async fn listed_count(&self, ctx: &AppContext) -> FieldResult<scalars::U64> {
        let conn = ctx.shared.db.get()?;
        listings::table
            .inner_join(metadatas::table.on(metadatas::address.eq(listings::metadata)))
            .inner_join(
                metadata_collection_keys::table
                    .on(metadata_collection_keys::metadata_address.eq(metadatas::address)),
            )
            .inner_join(
                auction_houses::table.on(listings::auction_house.eq(auction_houses::address)),
            )
            .filter(metadata_collection_keys::collection_address.eq(self.0.mint_address.clone()))
            .filter(listings::auction_house.ne(pubkeys::OPENSEA_AUCTION_HOUSE.to_string()))
            .filter(auction_houses::treasury_mint.eq("So11111111111111111111111111111111111111112"))
            .filter(metadata_collection_keys::verified.eq(true))
            .filter(listings::purchase_id.is_null())
            .filter(listings::canceled_at.is_null())
            .count()
            .get_result::<i64>(&conn)
            .context("Failed to load collection active listing count")?
            .try_into()
            .context("Collection listed count was too big to convert to U64")
            .map_err(Into::into)
    }

    #[graphql(description = "Count of NFTs in the collection.")]
    async fn nft_count(&self, context: &AppContext) -> FieldResult<Option<scalars::I64>> {
        Ok(context
            .collection_nft_count_loader
            .load(self.0.mint_address.clone().into())
            .await?
            .map(|dataloaders::collection::CollectionNftCount(nft_count)| nft_count))
    }

    #[graphql(
        description = "Total of all sales of all NFTs in the collection over all time, in lamports."
    )]
    pub async fn volume_total(&self, ctx: &AppContext) -> FieldResult<Option<scalars::U64>> {
        let conn = ctx.shared.db.get()?;

        let total_volume: Option<models::CollectionVolume> = sql_query(
            "SELECT VOLUME
                        FROM (
                            ( SELECT SUM(PURCHASES.PRICE) AS VOLUME
                            FROM PURCHASES
                        INNER JOIN METADATA_COLLECTION_KEYS ON METADATA_COLLECTION_KEYS.METADATA_ADDRESS = PURCHASES.METADATA
                            WHERE METADATA_COLLECTION_KEYS.COLLECTION_ADDRESS = $1
                            AND METADATA_COLLECTION_KEYS.VERIFIED = TRUE)
                        UNION
                            ( SELECT SUM(PURCHASES.PRICE) AS VOLUME
                            FROM PURCHASES
                            INNER JOIN ME_METADATA_COLLECTIONS ON ME_METADATA_COLLECTIONS.METADATA_ADDRESS = PURCHASES.METADATA
                            WHERE ME_METADATA_COLLECTIONS.COLLECTION_ID::text = $1)) AS VOLUME_TABLE
            LIMIT 1;",
        )
        .bind::<Text, _>(self.0.mint_address.clone())
        .load(&conn)
        .context("Failed to load collection volume total")?
        .first()
        .cloned();

        Ok(total_volume
            .map(|models::CollectionVolume { volume }| volume.to_u64().unwrap_or_default().into()))
    }

    #[graphql(deprecated = "use `nft { address }`")]
    pub fn address(&self) -> &str {
        &self.0.address
    }

    #[graphql(deprecated = "use `nft { name }`")]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    #[graphql(deprecated = "use `nft { seller_fee_basis_points }`")]
    pub fn seller_fee_basis_points(&self) -> i32 {
        self.0.seller_fee_basis_points
    }

    #[graphql(deprecated = "use `nft { mint_address }`")]
    pub fn mint_address(&self) -> &str {
        &self.0.mint_address
    }

    #[graphql(deprecated = "use `nft { token_account_address }`")]
    pub fn token_account_address(&self) -> &str {
        &self.0.token_account_address
    }

    #[graphql(deprecated = "use `nft { primary_sale_happened }`")]
    pub fn primary_sale_happened(&self) -> bool {
        self.0.primary_sale_happened
    }

    #[graphql(deprecated = "use `nft { update_authority_address }`")]
    pub fn update_authority_address(&self) -> &str {
        &self.0.update_authority_address
    }

    #[graphql(deprecated = "use `nft { description }`")]
    pub fn description(&self) -> &str {
        &self.0.description
    }

    #[graphql(deprecated = "use `nft { category }`")]
    pub fn category(&self) -> &str {
        &self.0.category
    }

    #[graphql(deprecated = "use `nft { parser }`")]
    pub fn parser(&self) -> Option<&str> {
        self.0.model.as_deref()
    }

    #[graphql(deprecated = "use `nft { image }`")]
    pub fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
        let url = Url::parse(&self.0.image);
        let id = if let Ok(ref url) = url {
            AssetIdentifier::new(url)
        } else {
            return Ok(self.0.image.clone());
        };

        let width = ImageSize::from(width.unwrap_or(ImageSize::XSmall as i32));
        let width_str = (width as i32).to_string();

        Ok(
            proxy_url(&ctx.shared.asset_proxy, &id, Some(("width", &*width_str)))?
                .map_or_else(|| self.0.image.clone(), |u| u.to_string()),
        )
    }

    #[graphql(deprecated = "use `nft { animation_url }`")]
    pub fn animation_url(&self) -> Option<&str> {
        self.0.animation_url.as_deref()
    }

    #[graphql(deprecated = "use `nft { external_url }`")]
    pub fn external_url(&self) -> Option<&str> {
        self.0.external_url.as_deref()
    }

    #[graphql(deprecated = "use `nft { creators }`")]
    pub async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        ctx.nft_creators_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { attributes }`")]
    pub async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        ctx.nft_attributes_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { owner }`")]
    pub async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        ctx.nft_owner_loader
            .load(self.0.mint_address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { ah_listings_loader }`")]
    pub async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<AhListing>> {
        ctx.ah_listings_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { purchases }`")]
    pub async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<Purchase>> {
        ctx.purchases_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { offers }`")]
    pub async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<Offer>> {
        ctx.offers_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { files }`")]
    pub async fn files(&self, ctx: &AppContext) -> FieldResult<Vec<NftFile>> {
        ctx.nft_files_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { collection }`")]
    pub async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<Collection>> {
        ctx.nft_collection_loader
            .load(self.0.address.clone().into())
            .await
            .map_err(Into::into)
    }

    #[graphql(deprecated = "use `nft { created_at }`")]
    pub async fn created_at(&self, ctx: &AppContext) -> FieldResult<Option<DateTime<Utc>>> {
        if let Some(slot) = self.0.slot {
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

impl TryFrom<models::Nft> for Collection {
    type Error = <Nft as TryFrom<models::Nft>>::Error;

    fn try_from(value: models::Nft) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

// TODO: use collection identifier for the data loader instead of string
#[derive(Debug, Clone)]
pub struct CollectionIdentifier(pub String);

#[derive(Debug, Clone)]
pub struct CollectionTrend {
    pub collection: String,
    pub floor_price: U64,
    pub nft_count: i32,
    pub one_day_volume: U64,
    pub seven_day_volume: U64,
    pub thirty_day_volume: U64,
    pub one_day_sales_count: U64,
    pub seven_day_sales_count: U64,
    pub thirty_day_sales_count: U64,
    pub prev_one_day_volume: U64,
    pub prev_seven_day_volume: U64,
    pub prev_thirty_day_volume: U64,
    pub prev_one_day_sales_count: U64,
    pub prev_seven_day_sales_count: U64,
    pub prev_thirty_day_sales_count: U64,
    pub prev_one_day_floor_price: U64,
    pub prev_seven_day_floor_price: U64,
    pub prev_thirty_day_floor_price: U64,
    pub one_day_volume_change: i32,
    pub seven_day_volume_change: i32,
    pub thirty_day_volume_change: i32,
    pub one_day_floor_price_change: i32,
    pub seven_day_floor_price_change: i32,
    pub thirty_day_floor_price_change: i32,
    pub one_day_sales_count_change: i32,
    pub seven_day_sales_count_change: i32,
    pub thirty_day_sales_count_change: i32,
    pub one_day_marketcap: U64,
    pub seven_day_marketcap: U64,
    pub thirty_day_marketcap: U64,
    pub one_day_marketcap_change: i32,
    pub seven_day_marketcap_change: i32,
    pub thirty_day_marketcap_change: i32,
}

impl<'a> TryFrom<models::CollectionTrend> for CollectionTrend {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CollectionTrend {
            collection,
            floor_price,
            nft_count,
            one_day_volume,
            seven_day_volume,
            thirty_day_volume,
            one_day_sales_count,
            seven_day_sales_count,
            thirty_day_sales_count,
            prev_one_day_volume,
            prev_seven_day_volume,
            prev_thirty_day_volume,
            prev_one_day_sales_count,
            prev_seven_day_sales_count,
            prev_thirty_day_sales_count,
            prev_one_day_floor_price,
            prev_seven_day_floor_price,
            prev_thirty_day_floor_price,
            one_day_volume_change,
            seven_day_volume_change,
            thirty_day_volume_change,
            one_day_floor_price_change,
            seven_day_floor_price_change,
            thirty_day_floor_price_change,
            one_day_sales_count_change,
            seven_day_sales_count_change,
            thirty_day_sales_count_change,
            one_day_marketcap,
            one_day_marketcap_change,
            seven_day_marketcap,
            seven_day_marketcap_change,
            thirty_day_marketcap,
            thirty_day_marketcap_change,
        }: models::CollectionTrend,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            collection,
            floor_price: floor_price.try_into().unwrap_or_default(),
            nft_count: nft_count.try_into()?,
            one_day_volume: one_day_volume.try_into().unwrap_or_default(),
            seven_day_volume: seven_day_volume.try_into().unwrap_or_default(),
            thirty_day_volume: thirty_day_volume.try_into().unwrap_or_default(),
            one_day_sales_count: one_day_sales_count.try_into().unwrap_or_default(),
            seven_day_sales_count: seven_day_sales_count.try_into().unwrap_or_default(),
            thirty_day_sales_count: thirty_day_sales_count.try_into().unwrap_or_default(),
            prev_one_day_volume: prev_one_day_volume.try_into().unwrap_or_default(),
            prev_seven_day_volume: prev_seven_day_volume.try_into().unwrap_or_default(),
            prev_thirty_day_volume: prev_thirty_day_volume.try_into().unwrap_or_default(),
            prev_one_day_sales_count: prev_one_day_sales_count.try_into().unwrap_or_default(),
            prev_seven_day_sales_count: prev_seven_day_sales_count
                .to_u64()
                .unwrap_or_default()
                .into(),
            prev_thirty_day_sales_count: prev_thirty_day_sales_count
                .to_u64()
                .unwrap_or_default()
                .into(),
            prev_one_day_floor_price: prev_one_day_floor_price.try_into().unwrap_or_default(),
            prev_seven_day_floor_price: prev_seven_day_floor_price
                .to_u64()
                .unwrap_or_default()
                .into(),
            prev_thirty_day_floor_price: prev_thirty_day_floor_price
                .to_u64()
                .unwrap_or_default()
                .into(),
            one_day_volume_change: one_day_volume_change.try_into()?,
            seven_day_volume_change: seven_day_volume_change.try_into()?,
            thirty_day_volume_change: thirty_day_volume_change.try_into()?,
            one_day_floor_price_change: one_day_floor_price_change.try_into()?,
            seven_day_floor_price_change: seven_day_floor_price_change.try_into()?,
            thirty_day_floor_price_change: thirty_day_floor_price_change.try_into()?,
            one_day_sales_count_change: one_day_sales_count_change.try_into()?,
            seven_day_sales_count_change: seven_day_sales_count_change.try_into()?,
            thirty_day_sales_count_change: thirty_day_sales_count_change.try_into()?,
            one_day_marketcap: one_day_marketcap.try_into().unwrap_or_default(),
            seven_day_marketcap: seven_day_marketcap.try_into().unwrap_or_default(),
            thirty_day_marketcap: thirty_day_marketcap.try_into().unwrap_or_default(),
            one_day_marketcap_change: one_day_marketcap_change.try_into()?,
            seven_day_marketcap_change: seven_day_marketcap_change.try_into()?,
            thirty_day_marketcap_change: thirty_day_marketcap_change.try_into()?,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl CollectionTrend {
    pub fn floor_price(&self) -> U64 {
        self.floor_price
    }

    pub fn nft_count(&self) -> i32 {
        self.nft_count
    }

    pub fn one_day_volume(&self) -> U64 {
        self.one_day_volume
    }

    pub fn seven_day_volume(&self) -> U64 {
        self.seven_day_volume
    }

    pub fn thirty_day_volume(&self) -> U64 {
        self.thirty_day_volume
    }

    pub fn one_day_sales_count(&self) -> U64 {
        self.one_day_sales_count
    }

    pub fn seven_day_sales_count(&self) -> U64 {
        self.seven_day_sales_count
    }

    pub fn thirty_day_sales_count(&self) -> U64 {
        self.thirty_day_sales_count
    }

    pub fn prev_one_day_volume(&self) -> U64 {
        self.prev_one_day_volume
    }

    pub fn prev_seven_day_volume(&self) -> U64 {
        self.prev_seven_day_volume
    }

    pub fn prev_thirty_day_volume(&self) -> U64 {
        self.prev_thirty_day_volume
    }

    pub fn prev_one_day_sales_count(&self) -> U64 {
        self.prev_one_day_sales_count
    }

    pub fn prev_seven_day_sales_count(&self) -> U64 {
        self.prev_seven_day_sales_count
    }

    pub fn prev_thirty_day_sales_count(&self) -> U64 {
        self.prev_thirty_day_sales_count
    }

    pub fn prev_one_day_floor_price(&self) -> U64 {
        self.prev_one_day_floor_price
    }

    pub fn prev_seven_day_floor_price(&self) -> U64 {
        self.prev_seven_day_floor_price
    }

    pub fn prev_thirty_day_floor_price(&self) -> U64 {
        self.prev_thirty_day_floor_price
    }

    pub fn one_day_volume_change(&self) -> i32 {
        self.one_day_volume_change
    }

    pub fn seven_day_volume_change(&self) -> i32 {
        self.seven_day_volume_change
    }

    pub fn thirty_day_volume_change(&self) -> i32 {
        self.thirty_day_volume_change
    }

    pub fn one_day_floor_price_change(&self) -> i32 {
        self.one_day_floor_price_change
    }

    pub fn seven_day_floor_price_change(&self) -> i32 {
        self.seven_day_floor_price_change
    }

    pub fn thirty_day_floor_price_change(&self) -> i32 {
        self.thirty_day_floor_price_change
    }

    pub fn one_day_sales_count_change(&self) -> i32 {
        self.one_day_sales_count_change
    }

    pub fn seven_day_sales_count_change(&self) -> i32 {
        self.seven_day_sales_count_change
    }

    pub fn thirty_day_sales_count_change(&self) -> i32 {
        self.thirty_day_sales_count_change
    }

    pub fn one_day_marketcap(&self) -> U64 {
        self.one_day_marketcap
    }

    pub fn seven_day_marketcap(&self) -> U64 {
        self.one_day_marketcap
    }

    pub fn thirty_day_marketcap(&self) -> U64 {
        self.one_day_marketcap
    }

    pub fn one_day_marketcap_change(&self) -> i32 {
        self.one_day_marketcap_change
    }

    pub fn seven_day_marketcap_change(&self) -> i32 {
        self.one_day_marketcap_change
    }

    pub fn thirty_day_marketcap_change(&self) -> i32 {
        self.one_day_marketcap_change
    }

    pub async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<Collection>> {
        ctx.generic_collection_loader
            .load(self.collection.clone())
            .await
            .map_err(Into::into)
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
