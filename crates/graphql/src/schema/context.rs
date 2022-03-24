use dataloaders::{Batcher, Loader};
use objects::{
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    listing::{Bid, Listing},
    listing_receipt::ListingReceipt,
    nft::{Nft, NftAttribute, NftCreator, NftOwner},
    purchase_receipt::PurchaseReceipt,
    stats::{MarketStats, MintStats},
    store_creator::StoreCreator,
    storefront::Storefront,
};
use scalars::{markers::StoreConfig, PublicKey};

use super::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: Arc<Pool>,
    pub(crate) shared: Arc<SharedData>,

    // Data loaders
    pub auction_house_loader: Loader<PublicKey<AuctionHouse>, Option<AuctionHouse>>,
    pub listing_loader: Loader<PublicKey<Listing>, Option<Listing>>,
    pub listing_bids_loader: Loader<PublicKey<Listing>, Vec<Bid>>,
    pub listing_nfts_loader: Loader<PublicKey<Listing>, Vec<(usize, Nft)>>,
    pub market_stats_loader: Loader<PublicKey<StoreConfig>, Option<MarketStats>>,
    pub mint_stats_loader: Loader<PublicKey<AuctionHouse>, Option<MintStats>>,
    pub nft_attributes_loader: Loader<PublicKey<Nft>, Vec<NftAttribute>>,
    pub nft_creators_loader: Loader<PublicKey<Nft>, Vec<NftCreator>>,
    pub nft_owner_loader: Loader<PublicKey<Nft>, Option<NftOwner>>,
    pub storefront_loader: Loader<PublicKey<Storefront>, Option<Storefront>>,
    pub listing_receipts_loader: Loader<PublicKey<Nft>, Vec<ListingReceipt>>,
    pub purchase_receipts_loader: Loader<PublicKey<Nft>, Vec<PurchaseReceipt>>,
    pub bid_receipts_loader: Loader<PublicKey<Nft>, Vec<BidReceipt>>,
    pub store_creator_loader: Loader<PublicKey<StoreConfig>, Vec<StoreCreator>>,
    pub collection_loader: Loader<PublicKey<StoreCreator>, Vec<Nft>>,
}

impl juniper::Context for AppContext {}

impl AppContext {
    pub(crate) fn new(db_pool: Arc<Pool>, shared: Arc<SharedData>) -> AppContext {
        let batcher = Batcher::new(db_pool.clone());

        Self {
            auction_house_loader: Loader::new(batcher.clone()),
            listing_loader: Loader::new(batcher.clone()),
            listing_bids_loader: Loader::new(batcher.clone()),
            listing_nfts_loader: Loader::new(batcher.clone()),
            market_stats_loader: Loader::new(batcher.clone()),
            mint_stats_loader: Loader::new(batcher.clone()),
            nft_attributes_loader: Loader::new(batcher.clone()),
            nft_creators_loader: Loader::new(batcher.clone()),
            nft_owner_loader: Loader::new(batcher.clone()),
            storefront_loader: Loader::new(batcher.clone()),
            listing_receipts_loader: Loader::new(batcher.clone()),
            purchase_receipts_loader: Loader::new(batcher.clone()),
            bid_receipts_loader: Loader::new(batcher.clone()),
            store_creator_loader: Loader::new(batcher.clone()),
            collection_loader: Loader::new(batcher),
            db_pool,
            shared,
        }
    }
}
