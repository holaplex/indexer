use dataloaders::{
    collection::{CollectionFloorPrice, CollectionHoldersCount, CollectionNftCount},
    Batcher, Loader, TwitterBatcher,
};
use indexer_core::uuid::Uuid;
use objects::{
    ah_listing::AhListing,
    ah_offer::Offer as AhOffer,
    ah_purchase::Purchase as AhPurchase,
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    candy_machine::{
        CandyMachine, CandyMachineCollectionPda, CandyMachineConfigLine, CandyMachineCreator,
        CandyMachineEndSetting, CandyMachineGateKeeperConfig, CandyMachineHiddenSetting,
        CandyMachineWhitelistMintSetting,
    },
    genopets::{GenoHabitat, GenoRentalAgreement},
    graph_connection::GraphConnection,
    listing::{Bid, Listing},
    listing_receipt::ListingReceipt,
    nft::{Collection, Nft, NftActivity, NftAttribute, NftCreator, NftFile, NftOwner},
    profile::TwitterProfile,
    purchase_receipt::PurchaseReceipt,
    spl_governance::{
        Governance, GovernanceConfig, MultiChoice, Proposal, ProposalOption, ProposalV1,
        ProposalV2, Realm, RealmConfig, TokenOwnerRecord, VoteChoice, VoteRecordV2,
    },
    stats::{MarketStats, MintStats},
    store_creator::StoreCreator,
    storefront::Storefront,
    wallet::Wallet,
};
use scalars::{
    markers::{StoreConfig, TokenMint},
    PublicKey,
};

use super::prelude::*;

#[derive(Clone)]
pub struct AppContext {
    pub(crate) shared: Arc<SharedData>,

    // Postgres dataloaders
    pub ah_listing_loader: Loader<Uuid, Option<AhListing>>,
    pub ah_listings_loader: Loader<PublicKey<Nft>, Vec<AhListing>>,
    pub auction_house_loader: Loader<PublicKey<AuctionHouse>, Option<AuctionHouse>>,
    pub bid_receipt_loader: Loader<PublicKey<BidReceipt>, Option<BidReceipt>>,
    pub bid_receipts_loader: Loader<PublicKey<Nft>, Vec<BidReceipt>>,
    pub candy_machine_collection_pda_loader:
        Loader<PublicKey<CandyMachine>, Option<CandyMachineCollectionPda>>,
    pub candy_machine_config_line_loader:
        Loader<PublicKey<CandyMachine>, Vec<CandyMachineConfigLine>>,
    pub candy_machine_creator_loader: Loader<PublicKey<CandyMachine>, Vec<CandyMachineCreator>>,
    pub candy_machine_end_settings_loader:
        Loader<PublicKey<CandyMachine>, Option<CandyMachineEndSetting>>,
    pub candy_machine_gatekeeper_configs_loader:
        Loader<PublicKey<CandyMachine>, Option<CandyMachineGateKeeperConfig>>,
    pub candy_machine_hidden_settings_loader:
        Loader<PublicKey<CandyMachine>, Option<CandyMachineHiddenSetting>>,
    pub candy_machine_whitelist_mint_settings_loader:
        Loader<PublicKey<CandyMachine>, Option<CandyMachineWhitelistMintSetting>>,
    pub collection_count_loader: Loader<PublicKey<StoreCreator>, Option<i32>>,
    pub collection_floor_price_loader: Loader<PublicKey<Collection>, Option<CollectionFloorPrice>>,
    pub collection_holders_count_loader:
        Loader<PublicKey<Collection>, Option<CollectionHoldersCount>>,
    pub collection_loader: Loader<PublicKey<StoreCreator>, Vec<Nft>>,
    pub collection_nft_count_loader: Loader<PublicKey<Collection>, Option<CollectionNftCount>>,
    pub geno_habitat_loader: Loader<PublicKey<TokenMint>, Option<GenoHabitat>>,
    pub geno_rental_agreement_loader: Loader<PublicKey<GenoHabitat>, Option<GenoRentalAgreement>>,
    pub graph_connection_loader: Loader<PublicKey<GraphConnection>, Option<GraphConnection>>,
    pub listing_bids_loader: Loader<PublicKey<Listing>, Vec<Bid>>,
    pub listing_loader: Loader<PublicKey<Listing>, Option<Listing>>,
    pub listing_nfts_loader: Loader<PublicKey<Listing>, Vec<(usize, Nft)>>,
    pub listing_receipt_loader: Loader<PublicKey<ListingReceipt>, Option<ListingReceipt>>,
    pub listing_receipts_loader: Loader<PublicKey<Nft>, Vec<ListingReceipt>>,
    pub market_stats_loader: Loader<PublicKey<StoreConfig>, Option<MarketStats>>,
    pub mint_stats_loader: Loader<PublicKey<AuctionHouse>, Option<MintStats>>,
    pub nft_activities_loader: Loader<PublicKey<Nft>, Vec<NftActivity>>,
    pub nft_attributes_loader: Loader<PublicKey<Nft>, Vec<NftAttribute>>,
    pub nft_by_mint_loader: Loader<PublicKey<TokenMint>, Option<Nft>>,
    pub nft_collection_loader: Loader<PublicKey<Nft>, Option<Collection>>,
    pub nft_creators_loader: Loader<PublicKey<Nft>, Vec<NftCreator>>,
    pub nft_files_loader: Loader<PublicKey<Nft>, Vec<NftFile>>,
    pub nft_loader: Loader<PublicKey<Nft>, Option<Nft>>,
    pub nft_owner_loader: Loader<PublicKey<Nft>, Option<NftOwner>>,
    pub offer_loader: Loader<Uuid, Option<AhOffer>>,
    pub offers_loader: Loader<PublicKey<Nft>, Vec<AhOffer>>,
    pub purchase_loader: Loader<Uuid, Option<AhPurchase>>,
    pub purchase_receipt_loader: Loader<PublicKey<PurchaseReceipt>, Option<PurchaseReceipt>>,
    pub purchase_receipts_loader: Loader<PublicKey<Nft>, Vec<PurchaseReceipt>>,
    pub purchases_loader: Loader<PublicKey<Nft>, Vec<AhPurchase>>,
    pub spl_approve_vote_choices_loader: Loader<PublicKey<VoteRecordV2>, Vec<VoteChoice>>,
    pub spl_governance_config_loader: Loader<PublicKey<Governance>, Option<GovernanceConfig>>,
    pub spl_governance_loader: Loader<PublicKey<Governance>, Option<Governance>>,
    pub spl_proposal_loader: Loader<PublicKey<Proposal>, Option<Proposal>>,
    pub spl_proposal_multi_choice_loader: Loader<PublicKey<ProposalV2>, Option<MultiChoice>>,
    pub spl_proposal_options_loader: Loader<PublicKey<ProposalV2>, Vec<ProposalOption>>,
    pub spl_proposalv1_loader: Loader<PublicKey<ProposalV1>, Option<ProposalV1>>,
    pub spl_proposalv2_loader: Loader<PublicKey<ProposalV2>, Option<ProposalV2>>,
    pub spl_realm_config_loader: Loader<PublicKey<Realm>, Option<RealmConfig>>,
    pub spl_realm_loader: Loader<PublicKey<Realm>, Option<Realm>>,
    pub spl_token_owner_record_loader:
        Loader<PublicKey<TokenOwnerRecord>, Option<TokenOwnerRecord>>,
    pub spl_vote_record_token_owner_loader: Loader<PublicKey<Wallet>, Vec<TokenOwnerRecord>>,
    pub store_auction_houses_loader: Loader<PublicKey<StoreConfig>, Vec<AuctionHouse>>,
    pub store_creator_loader: Loader<PublicKey<StoreConfig>, Vec<StoreCreator>>,
    pub storefront_loader: Loader<PublicKey<Storefront>, Option<Storefront>>,
    pub twitter_handle_loader: Loader<PublicKey<Wallet>, Option<String>>,

    // Twitter dataloaders
    pub twitter_profile_loader: Loader<String, Option<TwitterProfile>, TwitterBatcher>,
}

impl juniper::Context for AppContext {}

impl AppContext {
    pub(crate) fn new(shared: Arc<SharedData>) -> AppContext {
        let batcher = Batcher::new(shared.db.clone());
        let twitter_batcher = TwitterBatcher::new(shared.clone());

        Self {
            shared,

            // Postgres dataloaders
            ah_listing_loader: Loader::new(batcher.clone()),
            ah_listings_loader: Loader::new(batcher.clone()),
            auction_house_loader: Loader::new(batcher.clone()),
            bid_receipt_loader: Loader::new(batcher.clone()),
            bid_receipts_loader: Loader::new(batcher.clone()),
            candy_machine_collection_pda_loader: Loader::new(batcher.clone()),
            candy_machine_config_line_loader: Loader::new(batcher.clone()),
            candy_machine_creator_loader: Loader::new(batcher.clone()),
            candy_machine_end_settings_loader: Loader::new(batcher.clone()),
            candy_machine_gatekeeper_configs_loader: Loader::new(batcher.clone()),
            candy_machine_hidden_settings_loader: Loader::new(batcher.clone()),
            candy_machine_whitelist_mint_settings_loader: Loader::new(batcher.clone()),
            collection_count_loader: Loader::new(batcher.clone()),
            collection_floor_price_loader: Loader::new(batcher.clone()),
            collection_loader: Loader::new(batcher.clone()),
            collection_holders_count_loader: Loader::new(batcher.clone()),
            collection_nft_count_loader: Loader::new(batcher.clone()),
            geno_habitat_loader: Loader::new(batcher.clone()),
            geno_rental_agreement_loader: Loader::new(batcher.clone()),
            graph_connection_loader: Loader::new(batcher.clone()),
            listing_bids_loader: Loader::new(batcher.clone()),
            listing_loader: Loader::new(batcher.clone()),
            listing_nfts_loader: Loader::new(batcher.clone()),
            listing_receipt_loader: Loader::new(batcher.clone()),
            listing_receipts_loader: Loader::new(batcher.clone()),
            market_stats_loader: Loader::new(batcher.clone()),
            mint_stats_loader: Loader::new(batcher.clone()),
            nft_activities_loader: Loader::new(batcher.clone()),
            nft_attributes_loader: Loader::new(batcher.clone()),
            nft_by_mint_loader: Loader::new(batcher.clone()),
            nft_collection_loader: Loader::new(batcher.clone()),
            nft_creators_loader: Loader::new(batcher.clone()),
            nft_files_loader: Loader::new(batcher.clone()),
            nft_loader: Loader::new(batcher.clone()),
            nft_owner_loader: Loader::new(batcher.clone()),
            offer_loader: Loader::new(batcher.clone()),
            offers_loader: Loader::new(batcher.clone()),
            purchase_loader: Loader::new(batcher.clone()),
            purchase_receipt_loader: Loader::new(batcher.clone()),
            purchase_receipts_loader: Loader::new(batcher.clone()),
            purchases_loader: Loader::new(batcher.clone()),
            spl_approve_vote_choices_loader: Loader::new(batcher.clone()),
            spl_governance_config_loader: Loader::new(batcher.clone()),
            spl_governance_loader: Loader::new(batcher.clone()),
            spl_proposal_loader: Loader::new(batcher.clone()),
            spl_proposal_multi_choice_loader: Loader::new(batcher.clone()),
            spl_proposal_options_loader: Loader::new(batcher.clone()),
            spl_proposalv1_loader: Loader::new(batcher.clone()),
            spl_proposalv2_loader: Loader::new(batcher.clone()),
            spl_realm_config_loader: Loader::new(batcher.clone()),
            spl_realm_loader: Loader::new(batcher.clone()),
            spl_token_owner_record_loader: Loader::new(batcher.clone()),
            spl_vote_record_token_owner_loader: Loader::new(batcher.clone()),
            store_auction_houses_loader: Loader::new(batcher.clone()),
            store_creator_loader: Loader::new(batcher.clone()),
            storefront_loader: Loader::new(batcher.clone()),
            twitter_handle_loader: Loader::new(batcher),

            // Twitter dataloaders
            twitter_profile_loader: Loader::new(twitter_batcher),
        }
    }

    #[inline]
    pub(crate) async fn wallet(&self, address: PublicKey<Wallet>) -> Result<Wallet> {
        let handle = self.twitter_handle_loader.load(address.clone()).await?;
        Ok(Wallet::new(address, handle))
    }
}
