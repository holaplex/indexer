use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dataloader::{non_cached::Loader, BatchFn};
use indexer_core::{
    db::{
        models,
        tables::{
            attributes, auction_caches, auction_datas, auction_datas_ext, auction_houses, bids,
            listing_metadatas, metadata_creators, metadata_jsons, metadatas, store_config_jsons,
            storefronts, token_accounts,
        },
        Pool,
    },
    prelude::*,
};
use juniper::{
    graphql_value, EmptyMutation, EmptySubscription, FieldError, FieldResult, GraphQLInputObject,
    GraphQLObject, ParseScalarResult, ParseScalarValue, RootNode, Value,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;

#[derive(Debug, Clone)]
struct Creator {
    address: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Lamports(u64);

#[juniper::graphql_scalar(description = "Lamports")]
impl<S> GraphQLScalar for Lamports
where
    S: ScalarValue,
{
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<Lamports> {
        v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl TryFrom<i64> for Lamports {
    type Error = std::num::TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

#[juniper::graphql_object(Context = AppContext)]
impl Creator {
    fn address(&self) -> String {
        self.address.clone()
    }

    pub fn attribute_groups(&self, context: &AppContext) -> Vec<AttributeGroup> {
        let conn = context.db_pool.get().unwrap();

        let metadatas: Vec<String> = metadata_creators::table
            .select(metadata_creators::metadata_address)
            .filter(metadata_creators::creator_address.eq(self.address.clone()))
            .load(&conn)
            .unwrap();

        let metadata_attributes: Vec<models::MetadataAttribute> = attributes::table
            .select(attributes::all_columns)
            .filter(attributes::metadata_address.eq(any(metadatas)))
            .load(&conn)
            .unwrap();

        metadata_attributes
            .into_iter()
            .fold(
                HashMap::new(),
                |mut groups,
                 models::MetadataAttribute {
                     trait_type, value, ..
                 }| {
                    *groups
                        .entry(trait_type.unwrap().to_lowercase())
                        .or_insert_with(HashMap::new)
                        .entry(value)
                        .or_insert(0) += 1;

                    groups
                },
            )
            .into_iter()
            .map(|(name, vars)| AttributeGroup {
                name,
                variants: vars
                    .into_iter()
                    .map(|(name, count)| {
                        let name = name.map_or_else(String::new, Cow::into_owned);

                        AttributeVariant { name, count }
                    })
                    .collect(),
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, GraphQLObject)]
struct AttributeVariant {
    name: String,
    count: i32,
}

#[derive(Debug, GraphQLObject)]
struct AttributeGroup {
    name: String,
    variants: Vec<AttributeVariant>,
}

#[derive(GraphQLInputObject, Clone, Debug)]
#[graphql(description = "Filter on NFT attributes")]
struct AttributeFilter {
    trait_type: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, GraphQLObject)]
struct NftDetail {
    description: String,
    image: String,
}

#[derive(Debug, Clone, GraphQLObject)]
struct NftOwner {
    address: String,
}

#[derive(Debug, Clone)]
struct NftCreator {
    address: String,
    metadata_address: String,
    share: i32,
    verified: bool,
}

#[juniper::graphql_object(Context = AppContext)]
impl NftCreator {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn metadata_address(&self) -> String {
        self.metadata_address.clone()
    }

    pub fn share(&self) -> i32 {
        self.share
    }

    pub fn verified(&self) -> bool {
        self.verified
    }
}

impl<'a> From<models::MetadataCreator<'a>> for NftCreator {
    fn from(
        models::MetadataCreator {
            creator_address,
            metadata_address,
            share,
            verified,
        }: models::MetadataCreator,
    ) -> Self {
        Self {
            address: creator_address.into_owned(),
            metadata_address: metadata_address.into_owned(),
            share,
            verified,
        }
    }
}

type ListingRow = (
    String,                // address
    String,                // store_address
    Option<NaiveDateTime>, // ends_at
    Option<i32>,           // gap_time
    Option<NaiveDateTime>, // last_bid_time
);

#[derive(Debug, Clone)]
struct NftAttribute {
    metadata_address: String,
    value: String,
    trait_type: String,
}

#[juniper::graphql_object(Context = AppContext)]
impl NftAttribute {
    pub fn metadata_address(&self) -> String {
        self.metadata_address.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub fn trait_type(&self) -> String {
        self.trait_type.clone()
    }
}

impl<'a> From<models::MetadataAttribute<'a>> for NftAttribute {
    fn from(
        models::MetadataAttribute {
            metadata_address,
            value,
            trait_type,
            ..
        }: models::MetadataAttribute,
    ) -> Self {
        Self {
            metadata_address: metadata_address.into_owned(),
            value: value.unwrap().into_owned(),
            trait_type: trait_type.unwrap().into_owned(),
        }
    }
}

#[derive(Debug, Clone)]
struct Listing {
    address: String,
    store_address: String,
    ended: bool,
}

impl Listing {
    fn new(
        (address, store_address, ends_at, gap_time, last_bid_time): ListingRow,
        now: NaiveDateTime,
    ) -> FieldResult<Self> {
        Ok(Self {
            address,
            store_address,
            ended: indexer_core::util::get_end_info(
                ends_at,
                gap_time.map(|i| chrono::Duration::seconds(i.into())),
                last_bid_time,
                now,
            )?
            .1,
        })
    }
}

#[juniper::graphql_object(Context = AppContext)]
impl Listing {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn store_address(&self) -> String {
        self.store_address.clone()
    }

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub async fn storefront(&self, ctx: &AppContext) -> Option<Storefront> {
        let fut = ctx
            .storefront_loader
            .load(StorefrontAddress(self.store_address.clone()));
        let result = fut.await;

        result
    }

    pub async fn nfts(&self, ctx: &AppContext) -> Vec<Nft> {
        let fut = ctx.listing_nfts_loader.load(self.address.clone());
        let result = fut.await;

        result
    }

    pub async fn bids(&self, ctx: &AppContext) -> Vec<Bid> {
        let fut = ctx.listing_bids_loader.load(self.address.clone());
        let result = fut.await;

        result
    }
}

#[derive(Debug, Clone)]
struct Bid {
    listing_address: String,
    bidder_address: String,
    last_bid_time: String,
    last_bid_amount: Lamports,
    cancelled: bool,
}

#[juniper::graphql_object(Context = AppContext)]
impl Bid {
    pub fn listing_address(&self) -> String {
        self.listing_address.clone()
    }

    pub fn bidder_address(&self) -> String {
        self.bidder_address.clone()
    }

    pub fn last_bid_time(&self) -> String {
        self.last_bid_time.clone()
    }

    pub fn last_bid_amount(&self) -> Lamports {
        self.last_bid_amount
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }

    pub async fn listing(&self, ctx: &AppContext) -> Option<Listing> {
        let fut = ctx.listing_loader.load(self.listing_address.clone());
        let result = fut.await;

        result
    }
}

impl<'a> TryFrom<models::Bid<'a>> for Bid {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::Bid {
            listing_address,
            bidder_address,
            last_bid_time,
            last_bid_amount,
            cancelled,
            ..
        }: models::Bid,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            listing_address: listing_address.into_owned(),
            bidder_address: bidder_address.into_owned(),
            last_bid_time: last_bid_time.to_string(),
            last_bid_amount: last_bid_amount.try_into()?,
            cancelled,
        })
    }
}

#[derive(Debug, Clone)]
struct Wallet {
    address: String,
}

#[derive(Debug, Clone)]
struct Profile {
    handle: String,
    profile_image_url_lowres: String,
    profile_image_url_highres: String,
    banner_image_url: String,
}

#[derive(Debug, Deserialize)]
struct TwitterShowResponse {
    screen_name: String,
    profile_image_url_https: String,
    profile_banner_url: String,
}

#[derive(Debug, Deserialize)]
struct TwitterProfilePictureResponse {
    data: TwitterProfilePicture,
}

#[derive(Debug, Deserialize)]
struct TwitterProfilePicture {
    profile_image_url: String,
}

#[juniper::graphql_object(Context = AppContext)]
impl Profile {
    fn handle(&self) -> String {
        self.handle.clone()
    }

    fn profile_image_url_lowres(&self) -> String {
        self.profile_image_url_lowres.clone()
    }

    fn profile_image_url_highres(&self) -> String {
        self.profile_image_url_highres.clone()
    }

    fn banner_image_url(&self) -> String {
        self.banner_image_url.clone()
    }
}

impl From<(TwitterProfilePictureResponse, TwitterShowResponse)> for Profile {
    fn from(
        (profile_picture_response, show_response): (
            TwitterProfilePictureResponse,
            TwitterShowResponse,
        ),
    ) -> Self {
        Self {
            banner_image_url: show_response.profile_banner_url,
            handle: show_response.screen_name,
            profile_image_url_highres: profile_picture_response.data.profile_image_url,
            profile_image_url_lowres: show_response.profile_image_url_https,
        }
    }
}

#[juniper::graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        let db_conn = ctx.db_pool.get().unwrap();

        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)
            .filter(bids::bidder_address.eq(self.address.clone()))
            .order_by(bids::last_bid_time.desc())
            .load(&db_conn)
            .unwrap();

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
struct Nft {
    address: String,
    name: String,
    seller_fee_basis_points: i32,
    mint_address: String,
    primary_sale_happened: bool,
    description: String,
    image: String,
}

#[juniper::graphql_object(Context = AppContext)]
impl Nft {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn mint_address(&self) -> String {
        self.mint_address.clone()
    }

    pub fn primary_sale_happened(&self) -> bool {
        self.primary_sale_happened
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn image(&self) -> String {
        self.image.clone()
    }

    pub async fn creators(&self, ctx: &AppContext) -> Vec<NftCreator> {
        ctx.nft_creator_loader.load(self.address.clone()).await
    }

    pub async fn attributes(&self, ctx: &AppContext) -> Vec<NftAttribute> {
        ctx.nft_attribute_loader.load(self.address.clone()).await
    }

    pub async fn owner(&self, ctx: &AppContext) -> NftOwner {
        ctx.nft_owner_loader.load(self.address.clone()).await
    }

    
}

impl From<models::Nft> for Nft {
    fn from(
        models::Nft {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            primary_sale_happened,
            description,
            image,
        }: models::Nft,
    ) -> Self {
        Self {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            primary_sale_happened,
            description: description.unwrap_or_else(String::new),
            image: image.unwrap_or_else(String::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorefrontAddress(String);

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "A Metaplex storefront")]
pub struct Storefront {
    pub address: String,
    pub owner_address: String,
    pub subdomain: String,
    pub title: String,
    pub description: String,
    pub favicon_url: String,
    pub logo_url: String,
    pub banner_url: String,
}

impl<'a> From<models::Storefront<'a>> for Storefront {
    fn from(
        models::Storefront {
            address,
            owner_address,
            subdomain,
            title,
            description,
            favicon_url,
            logo_url,
            banner_url,
            ..
        }: models::Storefront,
    ) -> Self {
        Self {
            address: address.into_owned(),
            owner_address: owner_address.into_owned(),
            subdomain: subdomain.into_owned(),
            title: title.into_owned(),
            description: description.into_owned(),
            favicon_url: favicon_url.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.map_or_else(String::new, Cow::into_owned),
        }
    }
}

#[derive(Debug, Clone)]
struct Marketplace {
    subdomain: String,
    name: String,
    description: String,
    logo_url: String,
    banner_url: String,
    auction_house_address: String,
}

impl<'a> From<models::StoreConfigJson<'a>> for Marketplace {
    fn from(
        models::StoreConfigJson {
            config_address: _,
            name,
            description,
            logo_url,
            banner_url,
            subdomain,
            owner_address: _,
            auction_house_address,
            ..
        }: models::StoreConfigJson,
    ) -> Self {
        Self {
            subdomain: subdomain.into_owned(),
            name: name.into_owned(),
            description: description.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.into_owned(),
            auction_house_address: auction_house_address.into_owned(),
        }
    }
}

#[juniper::graphql_object(Context = AppContext)]
impl Marketplace {
    pub fn subdomain(&self) -> String {
        self.subdomain.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn logo_url(&self) -> String {
        self.logo_url.clone()
    }

    pub fn banner_url(&self) -> String {
        self.banner_url.clone()
    }

    pub fn auction_house_address(&self) -> String {
        self.auction_house_address.clone()
    }

    pub async fn auction_house(&self, context: &AppContext) -> Vec<AuctionHouse> {
        context
            .auction_house_loader
            .load(self.auction_house_address.clone())
            .await
    }
}

#[derive(Debug, Clone)]
struct AuctionHouse {
    address: String,
    authority: String,
    seller_fee_basis_points: i32,
    auction_house_fee_account: String,
}

#[juniper::graphql_object(Context = AppContext)]
impl AuctionHouse {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn authority(&self) -> String {
        self.authority.clone()
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn auction_house_fee_account(&self) -> String {
        self.auction_house_fee_account.clone()
    }
}
impl<'a> From<models::AuctionHouse<'a>> for AuctionHouse {
    fn from(
        models::AuctionHouse {
            address,
            treasury_mint: _,
            auction_house_treasury: _,
            treasury_withdrawal_destination: _,
            fee_withdrawal_destination: _,
            authority,
            creator: _,
            bump: _,
            treasury_bump: _,
            fee_payer_bump: _,
            seller_fee_basis_points,
            requires_sign_off: _,
            can_change_sale_price: _,
            auction_house_fee_account,
            ..
        }: models::AuctionHouse,
    ) -> Self {
        Self {
            address: address.into_owned(),
            authority: authority.into_owned(),
            seller_fee_basis_points: seller_fee_basis_points.into(),
            auction_house_fee_account: auction_house_fee_account.into_owned(),
        }
    }
}

pub struct QueryRoot {}

pub struct ListingBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<Listing>> for ListingBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Option<Listing>> {
        let now = Local::now().naive_utc();

        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

        let rows: Vec<ListingRow> = auction_caches::table
            .filter(auction_caches::auction_data.eq(any(keys)))
            .inner_join(
                auction_datas::table.on(auction_caches::auction_data.eq(auction_datas::address)),
            )
            .inner_join(
                auction_datas_ext::table
                    .on(auction_caches::auction_ext.eq(auction_datas_ext::address)),
            )
            .select((
                auction_datas::address,
                auction_caches::store_address,
                auction_datas::ends_at,
                auction_datas_ext::gap_tick_size,
                auction_datas::last_bid_time,
            ))
            .load(&conn)
            .unwrap();

        for listing in rows {
            let listing = Listing::new(listing, now)
                .map_err(|e| error!("Failed to load listing: {:?}", e))
                .ok();

            listing.map(|l| hash_map.insert(l.address.clone(), Some(l)));
        }

        hash_map
    }
}

pub struct StorefrontBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<StorefrontAddress, Option<Storefront>> for StorefrontBatcher {
    async fn load(
        &mut self,
        keys: &[StorefrontAddress],
    ) -> HashMap<StorefrontAddress, Option<Storefront>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

        let columns = (
            storefronts::owner_address,
            storefronts::subdomain,
            storefronts::title,
            storefronts::description,
            storefronts::favicon_url,
            storefronts::logo_url,
            storefronts::updated_at,
            storefronts::banner_url,
            storefronts::address,
        );

        let key_strs: Vec<_> = keys.iter().map(|k| &k.0).collect();

        let rows: Vec<models::Storefront> = storefronts::table
            .select(columns)
            .filter(storefronts::address.eq(any(key_strs)))
            .load(&conn)
            .unwrap();

        for storefront in rows {
            let storefront = Storefront::from(storefront);

            hash_map.insert(
                StorefrontAddress(storefront.address.clone()),
                Some(storefront),
            );
        }

        hash_map
    }
}

struct NftCreatorBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<NftCreator>> for NftCreatorBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<NftCreator>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for creator in addresses {
            hash_map.insert(creator.clone(), Vec::new());
        }

        let rows: Vec<models::MetadataCreator> = metadata_creators::table
            .filter(metadata_creators::metadata_address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, creator: models::MetadataCreator| {
                let creator = NftCreator::from(creator);
                acc.entry(creator.metadata_address.clone())
                    .and_modify(|creators| {
                        creators.push(creator);
                    });
                acc
            })
    }
}


struct NftOwnerBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, NftOwner> for NftOwnerBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, NftOwner> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for address in addresses { 
            hash_map.insert(address.clone(), NftOwner{address: String::from("")} );
        

        let mints: Vec<String> = metadatas::table
            .select(metadatas::mint_address)
            .filter(metadatas::address.eq(any(addresses)))
            .load(&conn)
            .unwrap();
            
        let owners: Vec<models::TokenAccount> = token_accounts::table
            .filter(token_accounts::mint_address.eq(any(&mints)))
            .load(&conn)
            .unwrap();
            if owners.len() > 0{
                hash_map.insert(address.clone(), NftOwner{address: owners[0].owner_address.to_string()});
            }
        }

        hash_map

    }
}

struct NftAttributeBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<NftAttribute>> for NftAttributeBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<NftAttribute>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for address in addresses {
            hash_map.insert(address.clone(), Vec::new());
        }

        let rows: Vec<models::MetadataAttribute> = attributes::table
            .filter(attributes::metadata_address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, attribute: models::MetadataAttribute| {
                let attribute = NftAttribute::from(attribute);
                acc.entry(attribute.metadata_address.clone())
                    .and_modify(|attributes| {
                        attributes.push(attribute);
                    });
                acc
            })
    }
}

pub struct ListingNftsBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<Nft>> for ListingNftsBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Vec<Nft>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), Vec::new());
        }

        let rows: Vec<(String, models::Nft)> = listing_metadatas::table
            .filter(listing_metadatas::listing_address.eq(any(keys)))
            .inner_join(
                metadatas::table.on(listing_metadatas::metadata_address.eq(metadatas::address)),
            )
            .inner_join(
                metadata_jsons::table
                    .on(listing_metadatas::metadata_address.eq(metadata_jsons::metadata_address)),
            )
            .select((
                listing_metadatas::listing_address,
                (
                    metadatas::address,
                    metadatas::name,
                    metadatas::seller_fee_basis_points,
                    metadatas::mint_address,
                    metadatas::primary_sale_happened,
                    metadata_jsons::description,
                    metadata_jsons::image,
                ),
            ))
            .load(&conn)
            .unwrap();

        rows.into_iter().fold(
            hash_map,
            |mut acc, (listing_address, nft): (String, models::Nft)| {
                acc.entry(listing_address).and_modify(|nfts| {
                    nfts.push(Nft::from(nft));
                });

                acc
            },
        )
    }
}

pub struct ListingBidsBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<Bid>> for ListingBidsBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Vec<Bid>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), Vec::new());
        }

        let rows: Vec<models::Bid> = bids::table
            .filter(bids::listing_address.eq(any(keys)))
            .order_by(bids::last_bid_time.desc())
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, bid: models::Bid| {
                Bid::try_from(bid)
                    .map(|bid| {
                        acc.entry(bid.listing_address.clone()).and_modify(|bids| {
                            bids.push(bid);
                        });
                    })
                    .ok();

                acc
            })
    }
}

struct AuctionHouseBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<AuctionHouse>> for AuctionHouseBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<AuctionHouse>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for auction_house in addresses {
            hash_map.insert(auction_house.clone(), Vec::new());
        }

        let rows: Vec<models::AuctionHouse> = auction_houses::table
            .filter(auction_houses::address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, ah: models::AuctionHouse| {
                let ah = AuctionHouse::from(ah);
                acc.entry(ah.address.clone()).and_modify(|ahs| {
                    ahs.push(ah);
                });
                acc
            })
    }
}

#[derive(Clone)]
pub struct AppContext {
    listing_loader: Loader<String, Option<Listing>, ListingBatcher>,
    listing_nfts_loader: Loader<String, Vec<Nft>, ListingNftsBatcher>,
    listing_bids_loader: Loader<String, Vec<Bid>, ListingBidsBatcher>,
    storefront_loader: Loader<StorefrontAddress, Option<Storefront>, StorefrontBatcher>,
    nft_creator_loader: Loader<String, Vec<NftCreator>, NftCreatorBatcher>,
    nft_attribute_loader: Loader<String, Vec<NftAttribute>, NftAttributeBatcher>,
    nft_owner_loader: Loader<String, NftOwner, NftOwnerBatcher>,
    auction_house_loader: Loader<String, Vec<AuctionHouse>, AuctionHouseBatcher>,
    db_pool: Arc<Pool>,
    twitter_bearer_token: Arc<String>,
}

impl AppContext {
    pub fn new(db_pool: Arc<Pool>, twitter_bearer_token: Arc<String>) -> AppContext {
        Self {
            listing_loader: Loader::new(ListingBatcher {
                db_pool: db_pool.clone(),
            }),
            listing_nfts_loader: Loader::new(ListingNftsBatcher {
                db_pool: db_pool.clone(),
            }),
            listing_bids_loader: Loader::new(ListingBidsBatcher {
                db_pool: db_pool.clone(),
            }),
            storefront_loader: Loader::new(StorefrontBatcher {
                db_pool: db_pool.clone(),
            }),
            nft_creator_loader: Loader::new(NftCreatorBatcher {
                db_pool: db_pool.clone(),
            }),
            nft_attribute_loader: Loader::new(NftAttributeBatcher {
                db_pool: db_pool.clone(),
            }),
            nft_owner_loader: Loader::new(NftOwnerBatcher {
                db_pool: db_pool.clone(),
            }),
            auction_house_loader: Loader::new(AuctionHouseBatcher {
                db_pool: db_pool.clone(),
            }),
            db_pool,
            twitter_bearer_token,
        }
    }
}

impl juniper::Context for AppContext {}

#[juniper::graphql_object(Context = AppContext)]
impl QueryRoot {
    async fn profile(
        &self,
        ctx: &AppContext,
        #[graphql(description = "Twitter handle")] handle: String,
    ) -> Option<Profile> {
        let twitter_bearer_token = &ctx.twitter_bearer_token;
        let http_client = HttpClient::new();

        let twitter_show_response = http_client
            .get("https://api.twitter.com/1.1/users/show.json")
            .header("Accept", "application/json")
            .query(&[("screen_name", handle.clone())])
            .bearer_auth(twitter_bearer_token)
            .send()
            .await
            .ok()?
            .json::<TwitterShowResponse>()
            .await
            .ok()?;

        let twitter_profile_picture_response = http_client
            .get(format!(
                "https://api.twitter.com/2/users/by/username/{}",
                handle.clone()
            ))
            .header("Accept", "application/json")
            .query(&[("user.fields", "profile_image_url")])
            .bearer_auth(twitter_bearer_token)
            .send()
            .await
            .ok()?
            .json::<TwitterProfilePictureResponse>()
            .await
            .ok()?;

        Some(Profile::from((
            twitter_profile_picture_response,
            twitter_show_response,
        )))
    }

    fn creator(
        &self,
        _context: &AppContext,
        #[graphql(description = "Address of creator")] address: String,
    ) -> Creator {
        Creator { address }
    }

    fn nfts(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on owner address")] owners: Option<Vec<String>>,
        #[graphql(description = "Filter on creator address")] creators: Option<Vec<String>>,
        #[graphql(description = "Filter on attributes")] attributes: Option<Vec<AttributeFilter>>,
    ) -> FieldResult<Vec<Nft>> {
        if owners.is_none() && creators.is_none() {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "owners: Vec<String>, creators: Vec<String>" }),
            ));
        }

        let conn = context.db_pool.get().context("failed to connect to db")?;

        let query = metadatas::table.into_boxed();
        let query = attributes.unwrap_or_else(Vec::new).into_iter().fold(
            query,
            |acc, AttributeFilter { trait_type, values }| {
                let sub = attributes::table
                    .select(attributes::metadata_address)
                    .filter(
                        attributes::trait_type
                            .eq(trait_type)
                            .and(attributes::value.eq(any(values))),
                    );

                acc.filter(metadatas::address.eq(any(sub)))
            },
        );

        let mint_addresses_subquery = token_accounts::table
            .select(token_accounts::mint_address)
            .filter(
                token_accounts::owner_address
                    .eq(any(owners.unwrap_or_else(Vec::new)))
                    .and(token_accounts::amount.eq(1)),
            );
        let metadata_addresses_subquery = metadata_creators::table
            .select(metadata_creators::metadata_address)
            .filter(metadata_creators::creator_address.eq(any(creators.unwrap_or_else(Vec::new))));

        let rows: Vec<models::Nft> = query
            .filter(
                metadatas::address
                    .eq(any(metadata_addresses_subquery))
                    .or(metadatas::mint_address.eq(any(mint_addresses_subquery))),
            )
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::seller_fee_basis_points,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .order_by(metadatas::name.desc())
            .load(&conn)
            .context("failed to load nft")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    fn wallet(
        &self,
        _context: &AppContext,
        #[graphql(description = "Address of NFT")] address: String,
    ) -> Option<Wallet> {
        Some(Wallet { address })
    }

    fn nft(
        &self,
        context: &AppContext,
        #[graphql(description = "Address of NFT")] address: String,
    ) -> Option<Nft> {
        let conn = context.db_pool.get().unwrap();
        let mut rows: Vec<models::Nft> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(metadatas::address.eq(address))
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::seller_fee_basis_points,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
    }

    #[graphql(description = "A storefront")]
    fn storefront(&self, context: &AppContext, subdomain: String) -> Option<Storefront> {
        let columns = (
            storefronts::owner_address,
            storefronts::subdomain,
            storefronts::title,
            storefronts::description,
            storefronts::favicon_url,
            storefronts::logo_url,
            storefronts::updated_at,
            storefronts::banner_url,
            storefronts::address,
        );

        let conn = context.db_pool.get().unwrap();
        let mut rows: Vec<models::Storefront> = storefronts::table
            .filter(storefronts::subdomain.eq(subdomain))
            .select(columns)
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
    }
    #[graphql(description = "A marketplace")]
    fn marketplace(&self, context: &AppContext, subdomain: String) -> Option<Marketplace> {
        let conn = context.db_pool.get().unwrap();
        let mut rows: Vec<models::StoreConfigJson> = store_config_jsons::table
            .filter(store_config_jsons::subdomain.eq(subdomain))
            .select(store_config_jsons::all_columns)
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
    }
}

impl QueryRoot {
    fn new() -> Self {
        Self {}
    }
}
pub type Schema =
    RootNode<'static, QueryRoot, EmptyMutation<AppContext>, EmptySubscription<AppContext>>;

pub fn create() -> Schema {
    Schema::new(
        QueryRoot::new(),
        EmptyMutation::new(),
        EmptySubscription::new(),
    )
}
