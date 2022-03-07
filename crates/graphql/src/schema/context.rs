use dataloaders::{Batcher, Loader};
use objects::{
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    listing::{Bid, Listing},
    listing_receipt::ListingReceipt,
    nft::{Nft, NftAttribute, NftCreator, NftOwner},
    store_creator::StoreCreator,
    storefront::Storefront,
};
use scalars::PublicKey;

use super::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: Arc<Pool>,
    pub twitter_bearer_token: Arc<String>,

    // Data loaders
    pub auction_house_loader: Loader<PublicKey, Option<AuctionHouse>>,
    pub listing_loader: Loader<PublicKey, Option<Listing>>,
    pub listing_bids_loader: Loader<PublicKey, Vec<Bid>>,
    pub listing_nfts_loader: Loader<PublicKey, Vec<Nft>>,
    pub nft_attributes_loader: Loader<PublicKey, Vec<NftAttribute>>,
    pub nft_creators_loader: Loader<PublicKey, Vec<NftCreator>>,
    pub nft_owner_loader: Loader<PublicKey, Option<NftOwner>>,
    pub storefront_loader: Loader<PublicKey, Option<Storefront>>,
    pub listing_receipts_loader: Loader<PublicKey, Vec<ListingReceipt>>,
    pub bid_receipts_loader: Loader<PublicKey, Vec<BidReceipt>>,
    pub store_creator_loader: Loader<PublicKey, Vec<StoreCreator>>,
}

impl juniper::Context for AppContext {}

impl AppContext {
    pub fn new(db_pool: Arc<Pool>, twitter_bearer_token: Arc<String>) -> AppContext {
        let batcher = Batcher::new(db_pool.clone());

        Self {
            auction_house_loader: Loader::new(batcher.clone()),
            listing_loader: Loader::new(batcher.clone()),
            listing_bids_loader: Loader::new(batcher.clone()),
            listing_nfts_loader: Loader::new(batcher.clone()),
            nft_attributes_loader: Loader::new(batcher.clone()),
            nft_creators_loader: Loader::new(batcher.clone()),
            nft_owner_loader: Loader::new(batcher.clone()),
            storefront_loader: Loader::new(batcher.clone()),
            listing_receipts_loader: Loader::new(batcher.clone()),
            bid_receipts_loader: Loader::new(batcher.clone()),
            store_creator_loader: Loader::new(batcher),
            db_pool,
            twitter_bearer_token,
        }
    }
}
