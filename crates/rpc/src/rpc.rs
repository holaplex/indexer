use std::collections::hash_map::Entry;

use indexer_core::{
    db::{
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
    subdomain: String,
    #[serde(rename = "storeTitle")]
    store_title: String,
    items: Vec<ListingItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListingItem {
    name: String,
    uri: String,
}

#[rpc]
pub trait Rpc {
    #[rpc(name = "getListings")]
    fn get_listings(&self) -> Result<Vec<Listing>>;
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

        let items: Vec<(String, String, String, String, String)> = listings::table
            .inner_join(listing_metadatas::table.inner_join(metadatas::table))
            .inner_join(storefronts::table)
            .select((
                listings::address,
                storefronts::subdomain,
                storefronts::title,
                metadatas::name,
                metadatas::uri,
            ))
            .order_by((listings::address, listing_metadatas::metadata_index))
            .load(&db)
            .map_err(internal_error("Failed to load listings"))?;

        let mut listings = HashMap::default();

        for (address, subdomain, store_title, name, uri) in items {
            match listings.entry(address.clone()) {
                Entry::Vacant(v) => {
                    v.insert(Listing {
                        address,
                        subdomain,
                        store_title,
                        items: vec![ListingItem { name, uri }],
                    });
                },
                Entry::Occupied(o) => {
                    let o = o.into_mut();
                    o.items.push(ListingItem { name, uri });
                },
            }
        }

        Ok(listings.into_values().collect())
    }
}
