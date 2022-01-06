use indexer_core::db::{
    models::{Metadata, Storefront},
    queries::{listings_triple_join, store_denylist},
    tables::{listing_metadatas, metadatas, storefronts},
    Pool, PooledConnection,
};
use jsonrpc_core::{Error, Result};
use jsonrpc_derive::rpc;

use crate::{
    prelude::*,
    rpc_models::{Listing, ListingItem, Storefront as RpcStorefront, Timestamp},
};

fn internal_error<E: Into<indexer_core::error::Error>>(
    msg: &'static str,
) -> impl FnOnce(E) -> Error {
    move |e| {
        error!("{}: {:?}", msg, e.into());
        Error::internal_error()
    }
}

#[rpc]
pub trait Rpc {
    #[rpc(name = "getListings")]
    fn get_listings(&self) -> Result<Vec<Listing>>;
    #[rpc(name = "getStorefronts")]
    fn get_storefronts(&self) -> Result<Vec<RpcStorefront>>;
    #[rpc(name = "getStoreCount")]
    fn get_store_count(&self) -> Result<i64>;
    #[rpc(name = "getStoreListings")]
    fn get_store_listings(&self, store_domain: String) -> Result<Vec<Listing>>;
    #[rpc(name = "getListingMetadatas")]
    fn get_listing_metadatas(&self, listing_address: String) -> Result<Vec<ListingItem>>;
}

pub struct Server {
    db_pool: Pool,
}

impl Server {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    fn db(&self) -> Result<PooledConnection> {
        self.db_pool
            .get()
            .map_err(internal_error("Failed to connect to the database"))
    }
}

impl Rpc for Server {
    fn get_listings(&self) -> Result<Vec<Listing>> {
        let db = self.db()?;

        listings_triple_join::load(|q| q, &db, Local::now().naive_utc())
            .map_err(internal_error("Failed to load listings"))
    }

    fn get_storefronts(&self) -> Result<Vec<RpcStorefront>> {
        let db = self.db()?;
        let rows: Vec<Storefront> = store_denylist::get_storefronts()
            .order_by(storefronts::owner_address)
            .load(&db)
            .map_err(internal_error("Failed to load storefronts"))?;

        Ok(rows
            .into_iter()
            .map(
                |Storefront {
                     owner_address,
                     subdomain,
                     title,
                     description,
                     favicon_url,
                     logo_url,
                     updated_at,
                 }| RpcStorefront {
                    owner_address: owner_address.into_owned(),
                    subdomain: subdomain.into_owned(),
                    title: title.into_owned(),
                    description: description.into_owned(),
                    favicon_url: favicon_url.into_owned(),
                    logo_url: logo_url.into_owned(),
                    updated_at: updated_at.map(Timestamp::from_utc),
                },
            )
            .collect())
    }

    fn get_store_count(&self) -> Result<i64> {
        let db = self.db()?;
        store_denylist::get_storefronts()
            .count()
            .get_result(&db)
            .map_err(internal_error("Failed to get store count"))
    }

    fn get_store_listings(&self, store_domain: String) -> Result<Vec<Listing>> {
        let db = self.db()?;

        listings_triple_join::load(
            |q| q.filter(storefronts::subdomain.eq(store_domain)),
            &db,
            Local::now().naive_utc(),
        )
        .map_err(internal_error("Failed to load store listings"))
    }

    fn get_listing_metadatas(&self, listing_address: String) -> Result<Vec<ListingItem>> {
        let db = self.db()?;
        let rows: Vec<Metadata> = listing_metadatas::table
            .inner_join(metadatas::table)
            .filter(listing_metadatas::listing_address.eq(listing_address))
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::symbol,
                metadatas::uri,
                metadatas::seller_fee_basis_points,
                metadatas::update_authority_address,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadatas::is_mutable,
                metadatas::edition_nonce,
            ))
            .order_by(listing_metadatas::metadata_index)
            .load(&db)
            .map_err(internal_error("Failed to load listing metadatas"))?;

        Ok(rows
            .into_iter()
            .map(
                |Metadata {
                     address,
                     name,
                     symbol: _,
                     uri,
                     seller_fee_basis_points: _,
                     update_authority_address: _,
                     mint_address: _,
                     primary_sale_happened: _,
                     is_mutable: _,
                     edition_nonce: _,
                 }| ListingItem {
                    address: address.into_owned(),
                    name: name.into_owned(),
                    uri: uri.into_owned(),
                },
            )
            .collect())
    }
}
