use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dataloader::{non_cached::Loader, BatchFn};
use indexer_core::{
    clap::App,
    db::{
        models,
        tables::{
            attributes, bids, listing_metadatas, listings, metadata_creators, metadata_jsons,
            metadatas, storefronts,
        },
        Pool,
    },
    prelude::*,
};
use juniper::{EmptyMutation, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode};
use reqwest::Client as HttpClient;
use serde::Deserialize;

#[derive(Debug, Clone)]
struct Creator {
    address: String,
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
                        .entry(trait_type)
                        .or_insert_with(HashMap::new)
                        .entry(value)
                        .or_insert(0) += 1;

                    groups
                },
            )
            .into_iter()
            .map(|(name, vars)| {
                let name = name.map_or_else(String::new, Cow::into_owned);

                AttributeGroup {
                    name,
                    variants: vars
                        .into_iter()
                        .map(|(name, count)| {
                            let name = name.map_or_else(String::new, Cow::into_owned);

                            AttributeVariant { name, count }
                        })
                        .collect(),
                }
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

#[derive(Debug, Clone)]
struct Listing {
    address: String,
    store_owner: String,
    ended: bool,
}

#[juniper::graphql_object(Context = AppContext)]
impl Listing {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn store_owner(&self) -> String {
        self.store_owner.clone()
    }

    pub fn ended(&self) -> bool {
        self.ended.clone()
    }

    pub async fn storefront(&self, ctx: &AppContext) -> Option<Storefront> {
        let fut = ctx.storefront_loader.load(self.store_owner.clone());
        let result = fut.await;

        result
    }

    pub async fn nfts(&self, ctx: &AppContext) -> Vec<Nft> {
        let fut = ctx.listing_nfts_loader.load(self.address.clone());
        let result = fut.await;

        result
    }
}

impl<'a> From<models::Listing<'a>> for Listing {
    fn from(
        models::Listing {
            address,
            store_owner,
            ended,
            ..
        }: models::Listing,
    ) -> Self {
        Self {
            address: address.into_owned(),
            store_owner: store_owner.into_owned(),
            ended,
        }
    }
}

#[derive(Debug, Clone)]
struct Bid {
    listing_address: String,
    bidder_address: String,
    last_bid_time: String,
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

    pub fn cancelled(&self) -> bool {
        self.cancelled.clone()
    }

    pub async fn listing(&self, ctx: &AppContext) -> Option<Listing> {
        let fut = ctx.listing_loader.load(self.listing_address.clone());
        let result = fut.await;

        result
    }
}

impl<'a> From<models::Bid<'a>> for Bid {
    fn from(
        models::Bid {
            listing_address,
            bidder_address,
            last_bid_time,
            cancelled,
            ..
        }: models::Bid,
    ) -> Self {
        Self {
            listing_address: listing_address.into_owned(),
            bidder_address: bidder_address.into_owned(),
            last_bid_time: last_bid_time.to_string(),
            cancelled,
        }
    }
}

#[derive(Debug, Clone)]
struct Wallet {
    address: String,
}

#[derive(Debug, Clone, GraphQLObject)]
struct Profile {
    handle: String,
    image_url: String,
    banner_url: String,
}

#[derive(Debug, Deserialize)]
struct TwitterUser {
    username: String,
    name: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct TwitterResponse<T> {
    data: T,
}

#[juniper::graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn bids(&self, ctx: &AppContext) -> Vec<Bid> {
        let db_conn = ctx.db_pool.get().unwrap();

        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)
            .filter(bids::bidder_address.eq(self.address.clone()))
            .order_by(bids::last_bid_time.desc())
            .load(&db_conn)
            .unwrap();

        rows.into_iter().map(Into::into).collect()
    }

    pub async fn profile(&self, ctx: &AppContext) -> Option<Profile> {
        let twitter_bearer_token = &ctx.twitter_bearer_token;
        // let solana_client = &ctx.solana_client;

        let http_client = HttpClient::new();

        // let (twitter_pubkey, _) = pubkeys::find_twitter_handle_address(&self.address);

        // let account_data = solana_client.get_account(&twitter_pubkey);

        // match account_data {
        //     Ok(_) => {
        //         println!("got data")
        //     },
        //     Err(_) => {
        //         println!("got error")
        //     }
        // }

        // println!("{:?} twitter account record", twitter_pubkey);

        let TwitterResponse {
            data: TwitterUser { username, .. },
        } = http_client
            .get("https://api.twitter.com/2/users/by/username/notjohnlestudio")
            .header("Accept", "application/json")
            .query(&[("user.fields", "username")])
            .bearer_auth(twitter_bearer_token)
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        Some(Profile {
            handle: username.to_string(),
            image_url: "profile_image_url".to_string(),
            banner_url: "https://todo".to_string(),
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
struct Nft {
    address: String,
    name: String,
    description: String,
    image: String,
}

impl From<models::Nft> for Nft {
    fn from(
        models::Nft {
            address,
            name,
            description,
            image,
        }: models::Nft,
    ) -> Self {
        Self {
            address,
            name,
            description: description.unwrap_or_else(String::new),
            image: image.unwrap_or_else(String::new),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "A Metaplex storefront")]
pub struct Storefront {
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

pub struct QueryRoot {}

pub struct ListingBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<Listing>> for ListingBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Option<Listing>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

        let rows: Vec<models::Listing> = listings::table
            .filter(listings::address.eq(any(keys)))
            .load(&conn)
            .unwrap();

        for listing in rows {
            let listing = Listing::from(listing);

            hash_map.insert(listing.address.clone(), Some(listing));
        }

        hash_map
    }
}

pub struct StorefrontBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<Storefront>> for StorefrontBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Option<Storefront>> {
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
        );

        let rows: Vec<models::Storefront> = storefronts::table
            .select(columns)
            .filter(storefronts::owner_address.eq(any(keys)))
            .load(&conn)
            .unwrap();

        for storefront in rows {
            let storefront = Storefront::from(storefront);

            hash_map.insert(storefront.owner_address.clone(), Some(storefront));
        }

        hash_map
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

#[derive(Clone)]
pub struct AppContext {
    listing_loader: Loader<String, Option<Listing>, ListingBatcher>,
    listing_nfts_loader: Loader<String, Vec<Nft>, ListingNftsBatcher>,
    storefront_loader: Loader<String, Option<Storefront>, StorefrontBatcher>,
    db_pool: Arc<Pool>,
    solana_client: Arc<solana_client::rpc_client::RpcClient>,
    twitter_bearer_token: Arc<String>,
}

impl AppContext {
    pub fn new(
        db_pool: Arc<Pool>,
        solana_client: Arc<solana_client::rpc_client::RpcClient>,
        twitter_bearer_token: Arc<String>,
    ) -> AppContext {
        Self {
            listing_loader: Loader::new(ListingBatcher {
                db_pool: db_pool.clone(),
            }),
            listing_nfts_loader: Loader::new(ListingNftsBatcher {
                db_pool: db_pool.clone(),
            }),
            storefront_loader: Loader::new(StorefrontBatcher {
                db_pool: db_pool.clone(),
            }),
            db_pool,
            solana_client,
            twitter_bearer_token,
        }
    }
}

impl juniper::Context for AppContext {}

#[juniper::graphql_object(Context = AppContext)]
impl QueryRoot {
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
        #[graphql(description = "Filter on creator address")] creators: Vec<String>,
        #[graphql(description = "Filter on attributes")] attributes: Option<Vec<AttributeFilter>>,
    ) -> Vec<Nft> {
        let conn = context.db_pool.get().unwrap();

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

        let rows: Vec<models::Nft> = query
            .filter(
                metadatas::address.eq(any(metadata_creators::table
                    .select(metadata_creators::metadata_address)
                    .filter(metadata_creators::creator_address.eq(any(creators))))),
            )
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .select((
                metadatas::address,
                metadatas::name,
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .order_by(metadatas::name.desc())
            .load(&conn)
            .unwrap();

        rows.into_iter().map(Into::into).collect()
    }

    fn wallet(
        &self,
        context: &AppContext,
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
