use indexer_core::db::{
    models,
    queries::{listings_triple_join, metadata_edition, store_denylist},
    tables::{bids, listing_metadatas, listings, metadata_creators, metadatas, storefronts},
    Pool, PooledConnection,
};
use jsonrpc_core::{Error, Result};
use jsonrpc_derive::rpc;

use crate::{
    prelude::*,
    rpc_models::{Listing, ListingDetails, ListingItem, Storefront, Timestamp},
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
    fn get_storefronts(&self) -> Result<Vec<Storefront>>;
    #[rpc(name = "getStoreCount")]
    fn get_store_count(&self) -> Result<i64>;
    #[rpc(name = "getStoreListings")]
    fn get_store_listings(&self, store_domain: String) -> Result<Vec<Listing>>;
    #[rpc(name = "getListingMetadatas")]
    fn get_listing_metadatas(&self, listing_address: String) -> Result<Vec<ListingItem>>;
    #[rpc(name = "getListingDetails")]
    fn get_listing_details(&self, listing_address: String) -> Result<ListingDetails>;
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

    fn get_storefronts(&self) -> Result<Vec<Storefront>> {
        let db = self.db()?;
        let rows: Vec<models::Storefront> = store_denylist::get_storefronts()
            .order_by(storefronts::owner_address)
            .load(&db)
            .map_err(internal_error("Failed to load storefronts"))?;

        Ok(rows
            .into_iter()
            .map(
                |models::Storefront {
                     owner_address,
                     subdomain,
                     title,
                     description,
                     favicon_url,
                     logo_url,
                     updated_at,
                 }| Storefront {
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
        let rows: Vec<models::Metadata> = listing_metadatas::table
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
                |models::Metadata {
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
                    extra: (),
                },
            )
            .collect())
    }

    fn get_listing_details(&self, listing_address: String) -> Result<ListingDetails> {
        let db = self.db()?;

        let listings: Vec<_> = unsafe {
            listings_triple_join::load_unfiltered(
                |q| q.filter(listings::address.eq(listing_address)),
                &db,
            )
        }
        .map_err(internal_error("Failed to load store listings"))?;

        let listing = if listings.len() == 1 {
            listings.into_iter().next().unwrap()
        } else {
            return Err(Error::invalid_params("Invalid listing address"));
        };

        ListingDetails::new(
            listing,
            |l| {
                Ok(bids::table
                    .filter(bids::listing_address.eq(&l.address))
                    .load::<models::Bid>(&db)
                    .context("Failed to load listing bids")?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            },
            |i| {
                Ok((
                    metadata_edition::load(&i.address, &db)?.map(Into::into),
                    metadata_creators::table
                        .filter(metadata_creators::metadata_address.eq(&i.address))
                        .load::<models::MetadataCreator>(&db)
                        .context("Failed to load metadata creators")?
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                ))
            },
        )
        .map_err(internal_error("Failed to pull listing details"))
    }
}
