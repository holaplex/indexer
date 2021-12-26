use indexer_core::{
    db::{
        models::{EditionOuterJoin, Listing, Metadata, MetadataCreator, Storefront},
        queries::listings_triple_join,
        tables::{
            editions, listing_metadatas, listings, master_editions, metadata_creators, metadatas,
            storefronts,
        },
        Pool, PooledConnection,
    },
    pubkeys::find_edition,
};
use jsonrpc_core::{Error, Result};
use jsonrpc_derive::rpc;
use solana_sdk::pubkey::Pubkey;

use crate::{
    prelude::*,
    rpc_models::{
        Creator, EditionOuterJoin as RpcEditionOuterJoin, Listing as RpcListing, ListingInfo,
        ListingItem, Storefront as RpcStorefront,
    },
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
    fn get_listings(&self) -> Result<Vec<RpcListing>>;
    #[rpc(name = "getStorefronts")]
    fn get_storefronts(&self) -> Result<Vec<RpcStorefront>>;
    #[rpc(name = "getStoreCount")]
    fn get_store_count(&self) -> Result<i64>;
    #[rpc(name = "getStoreListings")]
    fn get_store_listings(&self, store_domain: String) -> Result<Vec<RpcListing>>;
    #[rpc(name = "getListingMetadatas")]
    fn get_listing_metadatas(&self, listing_address: String) -> Result<Vec<ListingItem>>;
    #[rpc(name = "getListingInfo")]
    fn get_listing_info(&self, listing_address: String) -> Result<ListingInfo>;
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
    fn get_listings(&self) -> Result<Vec<RpcListing>> {
        let db = self.db()?;

        listings_triple_join::load(|q| q, &db, Local::now().naive_utc())
            .map_err(internal_error("Failed to load listings"))
    }

    fn get_storefronts(&self) -> Result<Vec<RpcStorefront>> {
        let db = self.db()?;
        let rows: Vec<Storefront> = storefronts::table
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
                 }| RpcStorefront {
                    owner_address: owner_address.into_owned(),
                    subdomain: subdomain.into_owned(),
                    title: title.into_owned(),
                    description: description.into_owned(),
                    favicon_url: favicon_url.into_owned(),
                    logo_url: logo_url.into_owned(),
                },
            )
            .collect())
    }

    fn get_store_count(&self) -> Result<i64> {
        let db = self.db()?;
        storefronts::table
            .count()
            .get_result(&db)
            .map_err(internal_error("Failed to get store count"))
    }

    fn get_store_listings(&self, store_domain: String) -> Result<Vec<RpcListing>> {
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

    fn get_listing_info(&self, listing_address: String) -> Result<ListingInfo> {
        let db = self.db()?;

        ///get metadata for the listing address
        let metadatas: Vec<Metadata> = listing_metadatas::table
            .inner_join(metadatas::table)
            .filter(
                listing_metadatas::listing_address
                    .eq(&listing_address)
                    .and(listing_metadatas::metadata_index.eq(0)),
            )
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
            .load(&db)
            .map_err(internal_error("Failed to load metadatas"))?;

        ///get listing row from 'Listings' table
        let listing: Vec<Listing> = listings::table
            .filter(listings::address.eq(listing_address))
            .load(&db)
            .map_err(internal_error("Failed to load listing"))?;

        ///get the creators of the listing
        let creators: Vec<MetadataCreator> = metadata_creators::table
            .filter(
                metadata_creators::metadata_address.eq(metadatas
                    .get(0)
                    .ok_or(Error::internal_error())?
                    .address
                    .to_string()),
            )
            .load(&db)
            .map_err(internal_error("Failed to load creators"))?;

        ///edition pubkey using the pda function
        let (edition_pubkey, _bump) = find_edition(&Pubkey::new(
            &bs58::decode((metadatas.get(0).ok_or(Error::internal_error())?.address).to_string())
                .into_vec()
                .unwrap(),
        ));

        ///Left join of 'editions' and 'master_edition' tables
        /// edition_pubkey could be 'master_edition' table address key or 'editions' table address key
        let edition: Vec<EditionOuterJoin> = master_editions::table
            .left_join(editions::table)
            .filter(
                master_editions::address
                    .eq(bs58::encode(edition_pubkey).into_string())
                    .or(editions::address.eq(bs58::encode(edition_pubkey).into_string())),
            )
            .select((
                master_editions::address,
                editions::address.nullable(),
                editions::edition.nullable(),
                master_editions::supply,
                master_editions::max_supply,
            ))
            .load(&db)
            .map_err(internal_error("Failed to load edition"))?;

        Ok(ListingInfo {
            address: metadatas
                .get(0)
                .ok_or(Error::internal_error())?
                .address
                .to_string(),
            name: metadatas
                .get(0)
                .ok_or(Error::internal_error())?
                .name
                .to_string(),
            uri: metadatas
                .get(0)
                .ok_or(Error::internal_error())?
                .uri
                .to_string(),
            ends_at: listing
                .get(0)
                .ok_or(Error::internal_error())?
                .ends_at
                .map(|e| e.to_string()),
            created_at: listing
                .get(0)
                .ok_or(Error::internal_error())?
                .created_at
                .to_string(),
            highest_bid: listing.get(0).ok_or(Error::internal_error())?.highest_bid,
            last_bid_time: listing
                .get(0)
                .ok_or(Error::internal_error())?
                .last_bid_time
                .map(|e| e.to_string()),
            edition: edition
                .into_iter()
                .map(
                    |EditionOuterJoin {
                         master_edition_address,
                         edition_address,
                         edition,
                         supply,
                         max_supply,
                     }| RpcEditionOuterJoin {
                        master_edition_address,
                        edition_address,
                        edition,
                        supply,
                        max_supply,
                    },
                )
                .collect(),
            creators: creators
                .into_iter()
                .map(
                    |MetadataCreator {
                         metadata_address: _,
                         creator_address,
                         share: _,
                         verified: _,
                     }| Creator {
                        creator_address: creator_address.to_string(),
                    },
                )
                .collect(),
        })
    }
}
