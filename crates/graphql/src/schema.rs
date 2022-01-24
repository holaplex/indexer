use indexer_core::{
    db::{
        models,
        tables::{metadata_creators, metadatas, storefronts},
        Pool,
    },
    prelude::*,
};
use juniper::{EmptyMutation, EmptySubscription, GraphQLObject, RootNode, FieldResult, FieldError};
use std::{collections::HashMap, hash::Hash};
use async_trait::async_trait;
use dataloader::non_cached::Loader;
use dataloader::BatchFn;

#[derive(Debug, Clone, GraphQLObject)]
struct NftDetail {
    title: String,
    description: String,
    image: String,
}

#[derive(Debug, Clone)]
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
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    pub async fn details(&self, ctx: &AppContext) -> Option<NftDetail> {
        let fut = ctx.nft_detail_loader.load(self.address.clone());

        Some(fut.await)
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


pub struct QueryRoot {
    db: Pool,
}

pub struct NftDetailBatcher;


impl BatchFn<String, NftDetail> for NftDetailBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, NftDetail> {
        let mut hash_map = HashMap::new();
        
        println!("{:?}", keys);

        for key in keys {
            hash_map.insert(key.clone(), NftDetail{ title: "foo".into(), description: "bar".into(), image: "image".into() });
        }

        hash_map
    }
}

#[derive(Clone)]
pub struct AppContext {
    nft_detail_loader: Loader<String, NftDetail, NftDetailBatcher>,
}

impl AppContext {
    pub fn new() -> AppContext {
        Self {
            nft_detail_loader: Loader::new(NftDetailBatcher),
        }
    }
}

impl juniper::Context for AppContext {}

#[juniper::graphql_object(Context = AppContext)]
impl QueryRoot {
    fn nfts(
        &self,
        #[graphql(description = "Filter on creator address")] creators: Option<Vec<String>>,
        #[graphql(description = "Filter on update authority addres")] update_authority: Option<
            Vec<String>,
        >,
    ) -> Vec<Nft> {
        let conn = self.db.get().unwrap();

        // Create mutable vector for all rows returned
        let mut all_rows: Vec<String> = Vec::new();

        // Iterate across creators passed into function
        for creator in creators.into_iter().flatten() {
            // Database stuff
            let mut rows: Vec<String> = metadata_creators::table
                .select(metadata_creators::metadata_address)
                .filter(metadata_creators::creator_address.eq(creator))
                .load(&conn)
                .unwrap();

            // Append found rows to all rows vector
            all_rows.append(&mut rows);
        }

        for ua in update_authority.into_iter().flatten() {
            // Database stuff
            let mut rows: Vec<String> = metadatas::table
                .select(metadatas::address)
                .filter(metadatas::update_authority_address.eq(ua))
                .load(&conn)
                .unwrap();

            // Append found rows to all rows vector
            all_rows.append(&mut rows);
        }

        // now find all nfts
        let rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(any(all_rows)))
            .load(&conn)
            .unwrap();

        // Cast Models::Metadata to Nft and return
        rows.into_iter().map(Into::into).collect()
    }

    fn nft(&self, #[graphql(description = "Address of NFT")] address: String) -> Option<Nft> {
        let conn = self.db.get().unwrap();
        let mut rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(address))
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
    }

    #[graphql(description = "A storefront")]
    fn storefront(&self, subdomain: String) -> Option<Storefront> {
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

        let conn = self.db.get().unwrap();
        let mut rows: Vec<models::Storefront> = storefronts::table
            .filter(storefronts::subdomain.eq(subdomain))
            .select(columns)
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
    }
}

pub type Schema = RootNode<'static, QueryRoot, EmptyMutation<AppContext>, EmptySubscription<AppContext>>;

pub fn create(db: Pool) -> Schema {
    Schema::new(
        QueryRoot { db },
        EmptyMutation::new(),
        EmptySubscription::new(),
    )
}
