use dataloader::non_cached::Loader;
use dataloaders::{
    auction_house::AuctionHouseBatcher,
    listing::{ListingBatcher, ListingBidsBatcher, ListingNftsBatcher},
    nft::{NftAttributeBatcher, NftCreatorBatcher, NftOwnerBatcher},
    storefront::StorefrontBatcher,
};
use objects::{
    auction_house::AuctionHouse,
    listing::{Bid, Listing},
    nft::{Nft, NftAttribute, NftCreator, NftOwner},
    storefront::{Storefront, StorefrontAddress},
};

use super::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub listing_loader: Loader<String, Option<Listing>, ListingBatcher>,
    pub listing_nfts_loader: Loader<String, Vec<Nft>, ListingNftsBatcher>,
    pub listing_bids_loader: Loader<String, Vec<Bid>, ListingBidsBatcher>,
    pub storefront_loader: Loader<StorefrontAddress, Option<Storefront>, StorefrontBatcher>,
    pub nft_creator_loader: Loader<String, Vec<NftCreator>, NftCreatorBatcher>,
    pub nft_attribute_loader: Loader<String, Vec<NftAttribute>, NftAttributeBatcher>,
    pub nft_owner_loader: Loader<String, Option<NftOwner>, NftOwnerBatcher>,
    pub auction_house_loader: Loader<String, Vec<AuctionHouse>, AuctionHouseBatcher>,
    pub db_pool: Arc<Pool>,
    pub twitter_bearer_token: Arc<String>,
}

impl juniper::Context for AppContext {}

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
