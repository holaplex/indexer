use std::collections::hash_map::Entry;

use indexer_core::{
    db::{
        models::{RpcGetListingsJoin, Storefront},
        tables::{listing_metadatas, listings, metadatas, storefronts},
        Pool, PooledConnection,
    },
    hash::HashMap,
};
use jsonrpc_core::{Error, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

fn internal_error<E: Into<indexer_core::error::Error>>(
    msg: &'static str,
) -> impl FnOnce(E) -> Error {
    move |e| {
        error!("{}: {:?}", msg, e.into());
        Error::internal_error()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Listing {
    address: String,
    ends_at: Option<String>,
    created_at: String,
    ended: bool,
    last_bid: Option<i64>,
    price_floor: Option<i64>,
    total_uncancelled_bids: Option<i32>,
    instant_sale_price: Option<i64>,
    subdomain: String,
    #[serde(rename = "storeTitle")]
    store_title: String,
    items: Vec<ListingItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storefronts {
    #[serde(rename = "ownerAddress")]
    owner_address: String,
    subdomain: String,
    title: String,
    description: String,
    #[serde(rename = "faviconUrl")]
    favicon_url: String,
    #[serde(rename = "logoUrl")]
    logo_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListingItem {
    address: String,
    name: String,
    uri: String,
}

#[rpc]
pub trait Rpc {
    #[rpc(name = "getListings")]
    fn get_listings(&self) -> Result<Vec<Listing>>;
    #[rpc(name = "getStorefronts")]
    fn get_stores(&self) -> Result<Vec<Storefronts>>;
    #[rpc(name = "getStoresCount")]
    fn get_stores_count(&self) -> Result<i64>;
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

        // TODO: figure out a less ugly way to perform this join
        let items: Vec<RpcGetListingsJoin> = listings::table
            .inner_join(listing_metadatas::table.inner_join(metadatas::table))
            .inner_join(storefronts::table)
            .select((
                listings::address,
                listings::ends_at,
                listings::created_at,
                listings::ended,
                listings::last_bid,
                listings::price_floor,
                listings::total_uncancelled_bids,
                listings::instant_sale_price,
                storefronts::subdomain,
                storefronts::title,
                metadatas::address,
                metadatas::name,
                metadatas::uri,
            ))
            .order_by((listings::address, listing_metadatas::metadata_index))
            .load(&db)
            .map_err(internal_error("Failed to load listings"))?;

        let mut listings = HashMap::default();

        for RpcGetListingsJoin {
            address,
            ends_at,
            created_at,
            ended,
            last_bid,
            price_floor,
            total_uncancelled_bids,
            instant_sale_price,
            subdomain,
            store_title,
            meta_address,
            name,
            uri,
        } in items
        {
            match listings.entry(address.clone()) {
                Entry::Vacant(v) => {
                    v.insert(Listing {
                        address,
                        ends_at: ends_at.map(|e| e.to_string()),
                        created_at: created_at.to_string(),
                        ended,
                        last_bid,
                        price_floor,
                        total_uncancelled_bids,
                        instant_sale_price,
                        subdomain,
                        store_title,
                        items: vec![ListingItem {
                            address: meta_address,
                            name,
                            uri,
                        }],
                    });
                },
                Entry::Occupied(o) => {
                    let o = o.into_mut();
                    o.items.push(ListingItem {
                        address: meta_address,
                        name,
                        uri,
                    });
                },
            }
        }

        Ok(listings.into_values().collect())
    }

    fn get_stores(&self) -> Result<Vec<Storefronts>> {
        let db = self.db()?;
        let items: Vec<Storefront> = storefronts::table
            .order_by(storefronts::owner_address)
            .load(&db)
            .map_err(internal_error("Failed to load storefronts"))?;
        let mut stores: Vec<Storefronts> = Vec::new();
        for Storefront {
            owner_address,
            subdomain,
            title,
            description,
            favicon_url,
            logo_url,
        } in items
        {
            stores.push(Storefronts {
                owner_address: owner_address.to_string(),
                subdomain: subdomain.to_string(),
                title: title.to_string(),
                description: description.to_string(),
                favicon_url: favicon_url.to_string(),
                logo_url: logo_url.to_string(),
            });
        }
        dbg!();
        println!("{}", stores.len());
        Ok(stores)
    }

    fn get_stores_count(&self) -> Result<i64> {
        let db = self.db()?;
        let count = storefronts::table.count().get_result(&db);
        Ok(count.unwrap())
    }
}
