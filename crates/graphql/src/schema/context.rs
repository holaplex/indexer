use dataloaders::{Batcher, Loader, TwitterBatcher};
use indexer_core::uuid::Uuid;
use objects::{
    ah_listing::Listing as AHListing,
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    graph_connection::GraphConnection,
    listing::{Bid, Listing},
    listing_receipt::ListingReceipt,
    nft::{Nft, NftActivity, NftAttribute, NftCreator, NftFile, NftOwner},
    offer::Offer,
    profile::TwitterProfile,
    purchase::Purchase,
    purchase_receipt::PurchaseReceipt,
    stats::{MarketStats, MintStats},
    store_creator::StoreCreator,
    storefront::Storefront,
};
use scalars::{markers::StoreConfig, PublicKey};

use super::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub(crate) shared: Arc<SharedData>,

    // Data loaders
    pub auction_house_loader: Loader<PublicKey<AuctionHouse>, Option<AuctionHouse>>,
    pub listing_loader: Loader<PublicKey<Listing>, Option<Listing>>,
    pub listing_bids_loader: Loader<PublicKey<Listing>, Vec<Bid>>,
    pub listing_nfts_loader: Loader<PublicKey<Listing>, Vec<(usize, Nft)>>,
    pub market_stats_loader: Loader<PublicKey<StoreConfig>, Option<MarketStats>>,
    pub mint_stats_loader: Loader<PublicKey<AuctionHouse>, Option<MintStats>>,
    pub nft_attributes_loader: Loader<PublicKey<Nft>, Vec<NftAttribute>>,
    pub nft_files_loader: Loader<PublicKey<Nft>, Vec<NftFile>>,
    pub nft_creators_loader: Loader<PublicKey<Nft>, Vec<NftCreator>>,
    pub nft_owner_loader: Loader<PublicKey<Nft>, Option<NftOwner>>,
    pub nft_activities_loader: Loader<PublicKey<Nft>, Vec<NftActivity>>,
    pub storefront_loader: Loader<PublicKey<Storefront>, Option<Storefront>>,
    pub graph_connection_loader: Loader<PublicKey<GraphConnection>, Option<GraphConnection>>,
    pub listing_receipts_loader: Loader<PublicKey<Nft>, Vec<ListingReceipt>>,
    pub listing_receipt_loader: Loader<PublicKey<ListingReceipt>, Option<ListingReceipt>>,
    pub ah_listings_loader: Loader<PublicKey<Nft>, Vec<AHListing>>,
    pub ah_listing_loader: Loader<Uuid, Option<AHListing>>,
    pub purchase_receipts_loader: Loader<PublicKey<Nft>, Vec<PurchaseReceipt>>,
    pub purchase_receipt_loader: Loader<PublicKey<PurchaseReceipt>, Option<PurchaseReceipt>>,
    pub purchases_loader: Loader<PublicKey<Nft>, Vec<Purchase>>,
    pub purchase_loader: Loader<Uuid, Option<Purchase>>,
    pub bid_receipts_loader: Loader<PublicKey<Nft>, Vec<BidReceipt>>,
    pub offers_loader: Loader<PublicKey<Nft>, Vec<Offer>>,
    pub offer_loader: Loader<Uuid, Option<Offer>>,
    pub bid_receipt_loader: Loader<PublicKey<BidReceipt>, Option<BidReceipt>>,
    pub store_creator_loader: Loader<PublicKey<StoreConfig>, Vec<StoreCreator>>,
    pub collection_count_loader: Loader<PublicKey<StoreCreator>, Option<i32>>,
    pub nft_loader: Loader<PublicKey<Nft>, Option<Nft>>,
    pub collection_loader: Loader<PublicKey<StoreCreator>, Vec<Nft>>,
    pub twitter_profile_loader: Loader<String, Option<TwitterProfile>, TwitterBatcher>,
}

impl juniper::Context for AppContext {}

impl AppContext {
    pub(crate) fn new(shared: Arc<SharedData>) -> AppContext {
        let batcher = Batcher::new(shared.db.clone());
        let twitter_batcher = TwitterBatcher::new(shared.twitter_bearer_token.clone());

        Self {
            auction_house_loader: Loader::new(batcher.clone()),
            listing_loader: Loader::new(batcher.clone()),
            listing_bids_loader: Loader::new(batcher.clone()),
            listing_nfts_loader: Loader::new(batcher.clone()),
            market_stats_loader: Loader::new(batcher.clone()),
            mint_stats_loader: Loader::new(batcher.clone()),
            nft_attributes_loader: Loader::new(batcher.clone()),
            nft_files_loader: Loader::new(batcher.clone()),
            nft_creators_loader: Loader::new(batcher.clone()),
            nft_owner_loader: Loader::new(batcher.clone()),
            graph_connection_loader: Loader::new(batcher.clone()),
            nft_activities_loader: Loader::new(batcher.clone()),
            storefront_loader: Loader::new(batcher.clone()),
            listing_receipts_loader: Loader::new(batcher.clone()),
            listing_receipt_loader: Loader::new(batcher.clone()),
            ah_listings_loader: Loader::new(batcher.clone()),
            ah_listing_loader: Loader::new(batcher.clone()),
            purchase_receipts_loader: Loader::new(batcher.clone()),
            purchase_receipt_loader: Loader::new(batcher.clone()),
            purchases_loader: Loader::new(batcher.clone()),
            purchase_loader: Loader::new(batcher.clone()),
            bid_receipts_loader: Loader::new(batcher.clone()),
            bid_receipt_loader: Loader::new(batcher.clone()),
            store_creator_loader: Loader::new(batcher.clone()),
            collection_count_loader: Loader::new(batcher.clone()),
            collection_loader: Loader::new(batcher.clone()),
            offers_loader: Loader::new(batcher.clone()),
            offer_loader: Loader::new(batcher.clone()),
            nft_loader: Loader::new(batcher),
            twitter_profile_loader: Loader::new(twitter_batcher),
            shared,
        }
    }
}
