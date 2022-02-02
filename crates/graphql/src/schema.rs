use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use dataloader::{non_cached::Loader, BatchFn};
use indexer_core::{
    db::{
        models,
        tables::{
            attributes::{self, metadata_address},
            metadata_creators::{self, creator_address},
            metadata_jsons, metadatas, storefronts,
        },
        Pool,
    },
    hash,
    prelude::*,
};
use juniper::{EmptyMutation, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode};

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

struct NftCreator {
    creators: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Nft {
    address: String,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: i32,
    update_authority_address: String,
    mint_address: String,
    primary_sale_happened: bool,
    is_mutable: bool,
}

#[juniper::graphql_object(Context = AppContext)]
impl Nft {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    pub async fn details(&self, ctx: &AppContext) -> Option<NftDetail> {
        let fut = ctx.nft_detail_loader.load(self.address.clone());
        let result = fut.await;

        result
    }

    pub async fn creators(&self, ctx: &AppContext) -> Option<NftCreator> {
        let fut = ctx.nft_creator_loader.load(self.address.clone());
        let result = fut.await;

        result
    }
}

impl<'a> From<models::Metadata<'a>> for Nft {
    fn from(
        models::Metadata {
            address,
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            update_authority_address,
            mint_address,
            primary_sale_happened,
            is_mutable,
            edition_nonce: _,
        }: models::Metadata,
    ) -> Self {
        Self {
            address: address.into_owned(),
            name: name.into_owned(),
            uri: uri.into_owned(),
            symbol: symbol.into_owned(),
            seller_fee_basis_points,
            update_authority_address: update_authority_address.into_owned(),
            mint_address: mint_address.into_owned(),
            primary_sale_happened,
            is_mutable,
        }
    }
}

#[derive(GraphQLObject)]
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

pub struct NftDetailBatcher {
    db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<NftDetail>> for NftDetailBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Option<NftDetail>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

        let nft_details: Vec<models::MetadataJson> = metadata_jsons::table
            .filter(metadata_jsons::metadata_address.eq(any(keys)))
            .load(&conn)
            .unwrap();

        for models::MetadataJson {
            metadata_address,
            image,
            description,
            ..
        } in nft_details
        {
            hash_map.insert(
                metadata_address.into_owned().to_string(),
                Some(NftDetail {
                    description: description.map_or_else(String::new, Cow::into_owned),
                    image: image.map_or_else(String::new, Cow::into_owned),
                }),
            );
        }

        hash_map
    }
}

#[async_trait]
impl BatchFn<String, Option<Nft>> for NftCreator {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Option<Nft>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        let nfts_creators: Vec<models::Metadata> = metadata_creators::table
            .filter(metadata_creators::metadata_address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        for models::MetadataCreator {
            metadata_address,
            creator_address,
        } in nfts_creators
        {
            hash_map
                .entry(&metadata_address)
                .or_insert_with(Vec::new)
                .push(creator_address);
        }
        hash_map
    }
}

#[derive(Clone)]
pub struct AppContext {
    nft_detail_loader: Loader<String, Option<NftDetail>, NftDetailBatcher>,
    nft_creator_loader: Loader<String, Option<NftDetail>, NftDetailBatcher>,
    db_pool: Arc<Pool>,
}

impl AppContext {
    pub fn new(db_pool: Arc<Pool>) -> AppContext {
        Self {
            nft_detail_loader: Loader::new(NftDetailBatcher {
                db_pool: db_pool.clone(),
            }),
            nft_creator_loader: Loader::new(NftDetailBatcher {
                db_pool: db_pool.clone(),
            }),
            db_pool,
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

        let query = metadatas::table.select(metadatas::all_columns).into_boxed();

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

        let rows: Vec<models::Metadata> = query
            .filter(
                metadatas::address.eq(any(metadata_creators::table
                    .select(metadata_creators::metadata_address)
                    .filter(metadata_creators::creator_address.eq(any(creators))))),
            )
            .order_by(metadatas::name.desc())
            .load(&conn)
            .unwrap();

        rows.into_iter().map(Into::into).collect()
    }

    fn nft(
        &self,
        context: &AppContext,
        #[graphql(description = "Address of NFT")] address: String,
    ) -> Option<Nft> {
        let conn = context.db_pool.get().unwrap();
        let mut rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(address))
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
