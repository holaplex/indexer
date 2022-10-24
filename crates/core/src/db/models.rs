//! Models to query and insert data according to the Diesel schema DSLs

// Queryable and Insertable are imported globally from diesel

use std::borrow::Cow;

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::sql_types::{
    Array, Bool, Int4, Int8, Nullable, Numeric, Text, Timestamp, Timestamptz, VarChar,
};
use uuid::Uuid;

#[allow(clippy::wildcard_imports)]
use super::schema::*;
use crate::db::custom_types::{
    EndSettingType, GovernanceAccountType, GovernanceAccountTypeEnum, InstructionExecutionFlags,
    InstructionExecutionFlagsEnum, ListingEventLifecycle, ListingEventLifecycleEnum,
    MintMaxVoteEnum, OfferEventLifecycle, OfferEventLifecycleEnum, OptionVoteResultEnum,
    PayoutOperationEnum, ProposalState, ProposalStateEnum, ProposalVoteType, ProposalVoteTypeEnum,
    TokenStandardEnum, TransactionExecutionStatusEnum, VoteRecordV2Vote, VoteRecordV2VoteEnum,
    VoteThresholdEnum, VoteThresholdType, VoteTippingEnum, VoteWeightV1, VoteWeightV1Enum,
    WhitelistMintMode,
};

/* MPL LISTING REWARDS */

/// A row in the `rewards_purchase_tickets` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
pub struct RewardsPurchaseTicket<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// reward center associated of the purchase ticket
    pub reward_center_address: Cow<'a, str>,
    /// the buyer of the nft
    pub buyer: Cow<'a, str>,
    /// the seller of the nft
    pub seller: Cow<'a, str>,
    /// the metadata of the nft purchased
    pub metadata: Cow<'a, str>,
    /// price of the nft
    pub price: i64,
    /// number of tokens sold
    pub token_size: i64,
    /// the date and time of the purchase
    pub created_at: NaiveDateTime,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `reward_centers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
pub struct RewardCenter<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// the mint of the token used as rewards
    pub token_mint: Cow<'a, str>,
    /// the auction house associated to the reward center
    pub auction_house: Cow<'a, str>,
    /// the bump of the pda
    pub bump: i16,
    /// Basis Points to determine reward ratio for seller
    pub seller_reward_payout_basis_points: i16,
    /// // Payout operation to consider when taking payout_numeral into account
    pub mathematical_operand: PayoutOperationEnum,
    /// Payout Divider for determining reward distribution to seller/buyer
    pub payout_numeral: i16,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `reward_payouts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct RewardPayout<'a> {
    /// Purchase ticket pubkey
    pub purchase_ticket: Cow<'a, str>,
    /// metadata address
    pub metadata: Cow<'a, str>,
    /// Reward center address
    pub reward_center: Cow<'a, str>,
    /// buyer wallet
    pub buyer: Cow<'a, str>,
    /// buyer reward
    pub buyer_reward: BigDecimal,
    /// seller wallet
    pub seller: Cow<'a, str>,
    /// seller reward
    pub seller_reward: BigDecimal,
    /// The timestamp when the reward payout was created.
    pub created_at: NaiveDateTime,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}
/// A row in `reward_payouts` table along with `buyer_twitter_handle` and `seller_twitter_handle`
#[derive(Debug, Clone, Queryable, QueryableByName)]
#[diesel(treat_none_as_null = true)]
pub struct ReadRewardPayout<'a> {
    /// Purchase ticket pubkey
    #[sql_type = "VarChar"]
    pub purchase_ticket: Cow<'a, str>,
    /// metadata address
    #[sql_type = "VarChar"]
    pub metadata: Cow<'a, str>,
    /// Reward center address
    #[sql_type = "VarChar"]
    pub reward_center: Cow<'a, str>,
    /// buyer wallet
    #[sql_type = "VarChar"]
    pub buyer: Cow<'a, str>,
    /// buyer twitter handle
    #[sql_type = "Nullable<Text>"]
    pub buyer_twitter_handle: Option<String>,
    /// buyer reward
    #[sql_type = "Numeric"]
    pub buyer_reward: BigDecimal,
    /// seller wallet
    #[sql_type = "VarChar"]
    pub seller: Cow<'a, str>,
    /// seller twitter handle
    #[sql_type = "Nullable<Text>"]
    pub seller_twitter_handle: Option<String>,
    /// seller reward
    #[sql_type = "Numeric"]
    pub seller_reward: BigDecimal,
    /// The timestamp when the reward payout was created.
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,
    /// The slot number of the most recent update for this account
    #[sql_type = "Int8"]
    pub slot: i64,
    /// The write version of the most recent update for this account
    #[sql_type = "Int8"]
    pub write_version: i64,
}

/// A row in the `rewards listings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "RewardCenter<'_>", foreign_key = "reward_center_address")]
pub struct RewardsListing<'a> {
    /// addres of listing account
    pub address: Cow<'a, str>,
    /// track initilization status of account
    pub is_initialized: bool,
    /// reward center of the listing
    pub reward_center_address: Cow<'a, str>,
    /// wallet selling the nft
    pub seller: Cow<'a, str>,
    /// nft being sold
    pub metadata: Cow<'a, str>,
    /// price of the nft
    pub price: i64,
    /// number of tokens sold by the listing
    pub token_size: i64,
    ///  the bump of the listing account
    pub bump: i16,
    /// date the listing was created
    pub created_at: NaiveDateTime,
    /// potentially when the listing was canceled
    pub canceled_at: Option<NaiveDateTime>,
    /// potentially purchase associated to the listing
    pub purchase_ticket: Option<Cow<'a, str>>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `rewards offers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "RewardCenter<'_>", foreign_key = "reward_center_address")]
pub struct RewardsOffer<'a> {
    /// address of the offer
    pub address: Cow<'a, str>,
    /// track initilization status of the offer
    pub is_initialized: bool,
    /// reward center offer made under
    pub reward_center_address: Cow<'a, str>,
    /// the wallet making the offer
    pub buyer: Cow<'a, str>,
    /// the nft the offer is placed on
    pub metadata: Cow<'a, str>,
    /// the offer amount
    pub price: i64,
    /// number of tokens offer made on
    pub token_size: i64,
    /// address bump of the pda
    pub bump: i16,
    /// when the offer was submitted
    pub created_at: NaiveDateTime,
    /// when the offer was canceled
    pub canceled_at: Option<NaiveDateTime>,
    /// the purchase associated to the offer in case of a sale
    pub purchase_ticket: Option<Cow<'a, str>>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/** MPL AUCTION HOUSE */
/// A row in the `bids` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "AuctionData<'_>", foreign_key = "listing_address")]
pub struct Bid<'a> {
    /// The auction being bid on
    pub listing_address: Cow<'a, str>,
    /// The wallet address of the bidding account
    pub bidder_address: Cow<'a, str>,
    /// The time the last bid by this user in this auction was placed
    pub last_bid_time: NaiveDateTime,
    /// The amount of the last bid
    pub last_bid_amount: i64,
    /// Whether the bid has been cancelled or redeemed
    pub cancelled: bool,
}

/// A row in the `editions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "MasterEdition<'_>", foreign_key = "parent_address")]
pub struct Edition<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The address of this edition's parent master edition
    pub parent_address: Cow<'a, str>,
    /// The ordinal of this edition
    pub edition: i64,
    /// Solana slot number
    pub slot: Option<i64>,
}

/// A row in the `listing_metadatas` table.  This is a join on `listings` and
/// `metadatas`
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "AuctionCache<'_>", foreign_key = "listing_address")]
#[belongs_to(parent = "Metadata<'_>", foreign_key = "metadata_address")]
pub struct ListingMetadata<'a> {
    /// The address of this record's listing
    pub listing_address: Cow<'a, str>,
    /// The address of this record's metadata
    pub metadata_address: Cow<'a, str>,
    /// The index of the metadata in the array of items for the listing
    pub metadata_index: i32,
}

/// A row in the `auction_caches` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
pub struct AuctionCache<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The storefront this auction cache belongs to
    pub store_address: Cow<'a, str>,
    /// The timestamp this auction cache was created at
    pub timestamp: NaiveDateTime,
    /// The address of the cached auction
    pub auction_data: Cow<'a, str>,
    /// The PDA of the cached auction's extended data
    pub auction_ext: Cow<'a, str>,
    /// The address of the cached auction's vault
    pub vault: Cow<'a, str>,
    /// The manager of the cached auction
    pub auction_manager: Cow<'a, str>,
}

/// A row in the `auction_datas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
pub struct AuctionData<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The timestamp this auction ends at, if applicable
    pub ends_at: Option<NaiveDateTime>,
    /// The authority of this auction
    pub authority: Cow<'a, str>,
    /// The item being auctioned
    pub token_mint: Cow<'a, str>,
    /// The amount of the highest bid, if applicable
    pub highest_bid: Option<i64>,
    /// The gap time of the auction, if applicable
    pub end_auction_gap: Option<NaiveDateTime>,
    /// The starting bid of the auction, if applicable
    pub price_floor: Option<i64>,
    /// The total number of live bids on this auction, if applicable
    pub total_uncancelled_bids: Option<i32>,
    /// The timestamp of the last bid, if applicable and the auction has bids
    pub last_bid_time: Option<NaiveDateTime>,
}

/// A row in the `auction_datas_ext` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[table_name = "auction_datas_ext"]
pub struct AuctionDataExt<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The minimum bid increase in percentage points during the ending gap of
    /// the auction, if applicable
    pub gap_tick_size: Option<i32>,
    /// The price of the listing, if an instant sale
    pub instant_sale_price: Option<i64>,
    /// The name of the listing
    pub name: Cow<'a, str>,
}

/// A row in the `master_editions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct MasterEdition<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The available printing supply of the master edition
    pub supply: i64,
    /// The maximum printing supply of the master edition, or `None` if it is
    /// unlimited
    pub max_supply: Option<i64>,
    /// Solana slot number
    pub slot: Option<i64>,
}

/// A row in the `metadata_creators` table.  This is a join on `metadatas` and
/// creator wallets.
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[diesel(treat_none_as_null = true)]
#[belongs_to(parent = "Metadata<'_>", foreign_key = "metadata_address")]
pub struct MetadataCreator<'a> {
    /// The address of this record's metadata
    pub metadata_address: Cow<'a, str>,
    /// The address of this record's creator wallet
    pub creator_address: Cow<'a, str>,
    /// The share of the creator, in percentage points
    pub share: i32,
    /// Whether this creator has verified this metadata
    pub verified: bool,
    /// position of creator in metadata creator array
    pub position: Option<i32>,
}

/// A row in the `metadatas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Metadata<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The name of this item
    pub name: Cow<'a, str>,
    /// The symbol for this item
    pub symbol: Cow<'a, str>,
    /// The URI for the off-chain item data
    pub uri: Cow<'a, str>,
    /// The royalty percentage of the creator, in basis points (0.01%, values
    /// range from 0-10,000)
    pub seller_fee_basis_points: i32,
    /// The authority over this item
    pub update_authority_address: Cow<'a, str>,
    /// The token address for this item
    pub mint_address: Cow<'a, str>,
    /// True if this item is in the secondary market.  Immutable once set.
    pub primary_sale_happened: bool,
    /// True if this item can be changed by the update authority
    pub is_mutable: bool,
    /// Metaplex isn't clear about what this is.  Assume reserved.
    pub edition_nonce: Option<i32>,
    /// edition pda derived from account
    pub edition_pda: Cow<'a, str>,
    /// Type of NFT token
    pub token_standard: Option<TokenStandardEnum>,
    /// Solana slot number
    pub slot: Option<i64>,
    /// Timestamp when the NFT was burned
    pub burned_at: Option<NaiveDateTime>,
}

/// A row in the `storefronts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Storefront<'a> {
    /// The address of this store's owner's wallet
    pub owner_address: Cow<'a, str>,
    /// The subdomain of the store on holaplex.com
    pub subdomain: Cow<'a, str>,
    /// The title of this store
    pub title: Cow<'a, str>,
    /// The description text for this store
    pub description: Cow<'a, str>,
    /// The file URL for this store's favicon
    pub favicon_url: Cow<'a, str>,
    /// The file URL for this store's logo
    pub logo_url: Cow<'a, str>,
    /// The timestamp this storefront was first uploaded to arweave
    pub updated_at: Option<NaiveDateTime>,
    /// The file URL for this store's banner
    pub banner_url: Option<Cow<'a, str>>,
    /// The address of this account
    ///
    /// **NOTE:** This is **NOT** the store owner's wallet!
    pub address: Cow<'a, str>,
}

/// Join of `metadatas`, `metadata_jsons` and `current_metadata_owners`  for an NFT
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct Nft {
    // Table metadata
    /// The address of this account
    #[sql_type = "VarChar"]
    pub address: String,

    /// The name of this item
    #[sql_type = "Text"]
    pub name: String,

    /// The royalty percentage of the creator, in basis points (0.01%, values
    /// range from 0-10,000)
    #[sql_type = "Int4"]
    pub seller_fee_basis_points: i32,

    /// The token address for this item
    #[sql_type = "VarChar"]
    pub mint_address: String,

    /// True if this item is in the secondary market.  Immutable once set.
    #[sql_type = "Bool"]
    pub primary_sale_happened: bool,

    /// The address of the Update Authority (for the Metadata PDA)
    #[sql_type = "VarChar"]
    pub update_authority_address: String,

    /// Metadata metadata_json uri
    #[sql_type = "Text"]
    pub uri: String,

    /// Solana slot number
    #[sql_type = "Nullable<Int8>"]
    pub slot: Option<i64>,

    // Table metadata_json
    /// Metadata description
    #[sql_type = "Nullable<Text>"]
    pub description: Option<String>,

    /// Metadata image URL
    #[sql_type = "Nullable<Text>"]
    pub image: Option<String>,

    /// Metadata animation URL
    #[sql_type = "Nullable<Text>"]
    pub animation_url: Option<String>,

    /// Metadata external URL
    #[sql_type = "Nullable<Text>"]
    pub external_url: Option<String>,

    /// Metadata Category
    #[sql_type = "Nullable<Text>"]
    pub category: Option<String>,

    /// Hint for what model the indexer parsed this NFT with
    #[sql_type = "Nullable<Text>"]
    pub model: Option<String>,

    // Table Current metadata owners
    /// TOken account address
    #[sql_type = "Text"]
    pub token_account_address: String,
}

/// Union of `listings` and `purchases` for an `NFTActivity`
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct NftActivity {
    /// The id of the activity
    #[sql_type = "diesel::sql_types::Uuid"]
    pub id: Uuid,

    /// The metadata associated of the activity
    #[sql_type = "VarChar"]
    pub metadata: String,

    /// The auction house activity generated from
    #[sql_type = "VarChar"]
    pub auction_house: String,

    /// The marketplace program pubkey
    #[sql_type = "VarChar"]
    pub marketplace_program: String,

    /// The price of listing or purchase
    #[sql_type = "Int8"]
    pub price: i64,

    /// Listing/Purchase created time
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,

    /// The wallet address asociated to the activity [seller, buyer]
    #[sql_type = "Array<VarChar>"]
    pub wallets: Vec<String>,

    /// The twitter handles asociated to each wallet [seller, buyer]
    #[sql_type = "Array<Nullable<Text>>"]
    pub wallet_twitter_handles: Vec<Option<String>>,

    /// Listing/Purchase created time
    #[sql_type = "Text"]
    pub activity_type: String,
}

/// A row in the `collection_trends` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[table_name = "collection_trends"]
pub struct CollectionTrend {
    /// Collection address or id
    pub collection: String,
    /// Collection 1 day volume
    #[column_name = "_1d_volume"]
    pub one_day_volume: BigDecimal,
    /// Collection 7 days volume
    #[column_name = "_7d_volume"]
    pub seven_day_volume: BigDecimal,
    /// Collection 30 days volume
    #[column_name = "_30d_volume"]
    pub thirty_day_volume: BigDecimal,
    /// Collection previous 1 day volume
    #[column_name = "_prev_1d_volume"]
    pub prev_one_day_volume: BigDecimal,
    /// Collection previous 7 days volume
    #[column_name = "_prev_7d_volume"]
    pub prev_seven_day_volume: BigDecimal,
    /// Collection previous 30 days volume
    #[column_name = "_prev_30d_volume"]
    pub prev_thirty_day_volume: BigDecimal,
    /// Collection 1 day sales count
    #[column_name = "_1d_sales_count"]
    pub one_day_sales_count: BigDecimal,
    /// Collection previous 1 day sales count
    #[column_name = "prev_1d_sales_count"]
    pub prev_one_day_sales_count: BigDecimal,
    /// Collection 7 days sales count
    #[column_name = "_7d_sales_count"]
    pub seven_day_sales_count: BigDecimal,
    /// Collection previous 7 days sales count
    #[column_name = "prev_7d_sales_count"]
    pub prev_seven_day_sales_count: BigDecimal,
    /// Collection 30 days sales count
    #[column_name = "_30d_sales_count"]
    pub thirty_day_sales_count: BigDecimal,
    /// Collection previous 30 days sales count
    #[column_name = "prev_30d_sales_count"]
    pub prev_thirty_day_sales_count: BigDecimal,
    /// Collection floor price
    pub floor_price: BigDecimal,
    /// Collection previous 1 day floor price
    #[column_name = "prev_1d_floor_price"]
    pub prev_one_day_floor_price: BigDecimal,
    /// Collection previous 7 days floor price
    #[column_name = "prev_7d_floor_price"]
    pub prev_seven_day_floor_price: BigDecimal,
    /// Collection previous 30 day floor price
    #[column_name = "prev_30d_floor_price"]
    pub prev_thirty_day_floor_price: BigDecimal,
    /// Collection 1 day volume change
    #[column_name = "_1d_volume_change"]
    pub one_day_volume_change: i64,
    /// Collection 7 days volume change
    #[column_name = "_7d_volume_change"]
    pub seven_day_volume_change: i64,
    /// Collection 30 days volume change
    #[column_name = "_30d_volume_change"]
    pub thirty_day_volume_change: i64,
    /// Collection 1 day floor price change
    #[column_name = "_1d_floor_price_change"]
    pub one_day_floor_price_change: i64,
    /// Collection 7 days floor price change
    #[column_name = "_7d_floor_price_change"]
    pub seven_day_floor_price_change: i64,
    /// Collection 30 day floor price change
    #[column_name = "_30d_floor_price_change"]
    pub thirty_day_floor_price_change: i64,
    /// Collection 1 day sales count change
    #[column_name = "_1d_sales_count_change"]
    pub one_day_sales_count_change: i64,
    /// Collection 7 days sales count change
    #[column_name = "_7d_sales_count_change"]
    pub seven_day_sales_count_change: i64,
    /// Collection 30 days sales count change
    #[column_name = "_30d_sales_count_change"]
    pub thirty_day_sales_count_change: i64,
    /// Collection 1 day marketcap
    #[column_name = "_1d_marketcap"]
    pub one_day_marketcap: BigDecimal,
    /// Collection prev 1 day marketcap
    #[column_name = "prev_1d_marketcap"]
    pub prev_one_day_marketcap: BigDecimal,
    /// Collection 7 day marketcap
    #[column_name = "_7d_marketcap"]
    pub seven_day_marketcap: BigDecimal,
    /// Collection prev 7 day marketcap
    #[column_name = "prev_7d_marketcap"]
    pub prev_seven_day_marketcap: BigDecimal,
    /// Collection 30 day marketcap
    #[column_name = "_30d_marketcap"]
    pub thirty_day_marketcap: BigDecimal,
    /// Collection prev 30 day marketcap
    #[column_name = "prev_30d_marketcap"]
    pub prev_thirty_day_marketcap: BigDecimal,
    /// Number of nfts in the collection
    pub nft_count: i64,
    /// Collection 1 day marketcap
    #[column_name = "_1d_marketcap_change"]
    pub one_day_marketcap_change: i64,
    /// Collection 7 day marketcap change
    #[column_name = "_7d_marketcap_change"]
    pub seven_day_marketcap_change: i64,
    /// Collection 30 day marketcap change
    #[column_name = "_30d_marketcap_change"]
    pub thirty_day_marketcap_change: i64,
}

/// Collection nfts/holders count
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct CollectionCount {
    /// Collection address or id
    #[sql_type = "Text"]
    pub collection: String,
    /// nfts/holders count
    #[sql_type = "Int8"]
    pub count: i64,
}

/// Collection floor price from the union of `collection_stats` and `me_collection_stats` table
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct CollectionFloorPrice {
    /// Collection address or id
    #[sql_type = "Text"]
    pub collection: String,
    /// Collection floor price
    #[sql_type = "Nullable<Int8>"]
    pub floor_price: Option<i64>,
}

/// Collection Volume
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct CollectionVolume {
    /// Collection Volume
    #[sql_type = "Numeric"]
    pub volume: BigDecimal,
}

/// Union of `listings`, `purchases`, `sell` and 'offers' for a `WalletActivity`
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct WalletActivity {
    /// The id of the activity
    #[sql_type = "diesel::sql_types::Uuid"]
    pub id: Uuid,

    /// The metadata associated of the activity
    #[sql_type = "VarChar"]
    pub metadata: String,

    /// The auction house activity generated from
    #[sql_type = "VarChar"]
    pub auction_house: String,

    /// The marketplace program pubkey
    #[sql_type = "VarChar"]
    pub marketplace_program: String,

    /// The price of listing or purchase
    #[sql_type = "Int8"]
    pub price: i64,

    /// Listing/Purchase created time
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,

    /// The wallet address asociated to the activity [seller, buyer]
    #[sql_type = "Array<VarChar>"]
    pub wallets: Vec<String>,

    /// The twitter handles asociated to each wallet [seller, buyer]
    #[sql_type = "Array<Nullable<Text>>"]
    pub wallet_twitter_handles: Vec<Option<String>>,

    /// Activity type - listing, purchase, sell or offer
    #[sql_type = "Text"]
    pub activity_type: String,
}

/// Join of `metadatas` `metadata_jsons` `store_creators` `current_metadata_owners` for an collection preview
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct SampleNft {
    // Table store_creators
    /// The store creators address
    #[sql_type = "VarChar"]
    pub creator_address: String,

    // Table metadata
    /// The address of this account
    #[sql_type = "VarChar"]
    pub address: String,

    /// The name of this item
    #[sql_type = "Text"]
    pub name: String,

    /// The royalty percentage of the creator, in basis points (0.01%, values
    /// range from 0-10,000)
    #[sql_type = "Int4"]
    pub seller_fee_basis_points: i32,

    /// The token address for this item
    #[sql_type = "VarChar"]
    pub mint_address: String,

    /// True if this item is in the secondary market.  Immutable once set.
    #[sql_type = "Bool"]
    pub primary_sale_happened: bool,

    /// The address of the Update Authority (for the Metadata PDA)
    #[sql_type = "VarChar"]
    pub update_authority_address: String,

    /// uri for metadata_json
    #[sql_type = "Text"]
    pub uri: String,

    // Table metadata_json
    /// Metadata description
    #[sql_type = "Nullable<Text>"]
    pub description: Option<String>,

    /// Metadata image URL
    #[sql_type = "Nullable<Text>"]
    pub image: Option<String>,

    /// Metadata animation URL
    #[sql_type = "Nullable<Text>"]
    pub animation_url: Option<String>,

    /// Metadata external URL
    #[sql_type = "Nullable<Text>"]
    pub external_url: Option<String>,

    /// Metadata category
    #[sql_type = "Nullable<Text>"]
    pub category: Option<String>,

    /// Hint for what model the indexer parsed this NFT with
    #[sql_type = "Nullable<Text>"]
    pub model: Option<String>,

    // Table Current metadata owners
    /// TOken account address
    #[sql_type = "Text"]
    pub token_account_address: String,
}

/// Join record for the RPC getListings query
#[derive(Debug, Clone, Queryable)]
pub struct ListingsTripleJoinRow {
    // Table `listings`
    /// Listing address
    pub address: String,
    /// Listing end time
    pub ends_at: Option<NaiveDateTime>,
    /// Listing created time
    pub created_at: NaiveDateTime,
    /// Listing ended flag
    pub ended: bool,
    /// Listing highest bid amount
    pub highest_bid: Option<i64>,
    /// The timestamp of the last bid on the listing, if available
    pub last_bid_time: Option<NaiveDateTime>,
    /// Listing price floor
    pub price_floor: Option<i64>,
    /// Listing bid count
    pub total_uncancelled_bids: Option<i32>,
    /// Listing instant-sale price
    pub instant_sale_price: Option<i64>,

    // Table `storefronts`
    /// Storefront subdomain
    pub subdomain: String,
    /// Storefront title
    pub store_title: String,
    /// Storefront logo
    pub logo_url: String,
    /// Storefront favicon
    pub favicon_url: String,

    // Table `metadatas`
    /// Metadata address
    pub meta_address: String,
    /// Metadata name
    pub name: String,
    /// Metadata URI
    pub uri: String,
    /// Listing has already been sold once
    pub primary_sale_happened: bool,
}

/// A row in the `metadata_jsons` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct MetadataJson<'a> {
    /// Metadata Address
    pub metadata_address: Cow<'a, str>,
    /// Metadata URI fingerprint - Cid for IPFS and ArTxid for Arweave
    pub fingerprint: Cow<'a, [u8]>,
    /// Metadata timestamp
    pub updated_at: NaiveDateTime,
    /// Metadata description
    pub description: Option<Cow<'a, str>>,
    /// Metadata Image URL
    pub image: Option<Cow<'a, str>>,
    /// Metadata Animation URL
    pub animation_url: Option<Cow<'a, str>>,
    /// Metadata External URL
    pub external_url: Option<Cow<'a, str>>,
    /// Metadata Category
    pub category: Option<Cow<'a, str>>,
    /// Metadata URI raw JSON
    pub raw_content: Cow<'a, serde_json::Value>,
    /// Model the JSON was parsed with
    pub model: Option<Cow<'a, str>>,
    /// The URI from which the data in this row was retrieved
    pub fetch_uri: Cow<'a, str>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
    /// Metadata name
    pub name: Option<Cow<'a, str>>,
}

/// A row in the `files` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct File<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// File URI attribute
    pub uri: Cow<'a, str>,
    /// File type attribute
    pub file_type: Cow<'a, str>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `attributes` table
#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "attributes"]
pub struct MetadataAttributeWrite<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// Attribute value
    pub value: Option<Cow<'a, str>>,
    /// Attribute trait type
    pub trait_type: Option<Cow<'a, str>>,
    /// Address of metadata first verified creator
    pub first_verified_creator: Option<Cow<'a, str>>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `attributes` table
#[derive(Debug, Clone, Queryable)]
pub struct MetadataAttribute<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// Attribute value
    pub value: Option<Cow<'a, str>>,
    /// Attribute trait type
    pub trait_type: Option<Cow<'a, str>>,
    /// Attribute generated id
    pub id: Uuid,
    /// Address of metadata first verified creator
    pub first_verified_creator: Option<Cow<'a, str>>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `files` table
#[derive(Debug, Clone, Queryable)]
pub struct MetadataFile<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// File uri
    pub uri: Cow<'a, str>,
    /// File type
    pub file_type: Cow<'a, str>,
    /// File generated id
    pub id: Uuid,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `metadata_collections` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct MetadataCollection<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// Collection name
    pub name: Option<Cow<'a, str>>,
    /// Collection family
    pub family: Option<Cow<'a, str>>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `store_configs` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct StoreConfig<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// Store settings URI
    pub settings_uri: Option<Cow<'a, str>>,
}

/// A row in the `whitelisted_creators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct WhitelistedCreator<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The wallet of the whitelisted creator
    pub creator_address: Cow<'a, str>,
    /// Whether or not the specified creator is actually whitelisted
    pub activated: bool,
}

/// A row in the `stores` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Store<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// Whether this is a public storefront
    ///
    /// When this flag is set, items with creators not in the set of active
    /// whitelisted creators can list on this storefront.
    pub public: bool,
    /// The derived address of this store's StoreConfig account
    pub config_address: Cow<'a, str>,
}

/// A row in the `settings_uri_jsons` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct StoreConfigJson<'a> {
    /// The address of the StoreConfig account this record refers to
    pub config_address: Cow<'a, str>,
    /// Storefront name
    pub name: Cow<'a, str>,
    /// Storefront description
    pub description: Cow<'a, str>,
    /// Storefront logo URL
    pub logo_url: Cow<'a, str>,
    /// Storefront banner URL
    pub banner_url: Cow<'a, str>,
    /// Storefront submain
    pub subdomain: Cow<'a, str>,
    /// Storefront owner address
    pub owner_address: Cow<'a, str>,
    /// Storefront address
    pub store_address: Option<Cow<'a, str>>,
}

/// A row in the `auction_houses` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct AuctionHouse<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// Auction House treasury mint address
    pub treasury_mint: Cow<'a, str>,
    /// Auction House treasury address
    pub auction_house_treasury: Cow<'a, str>,
    /// Treasury withdrawal address
    pub treasury_withdrawal_destination: Cow<'a, str>,
    /// Fee withdrawl address
    pub fee_withdrawal_destination: Cow<'a, str>,

    /// Auction House authority address
    pub authority: Cow<'a, str>,
    /// Auction House creator address
    pub creator: Cow<'a, str>,

    // Bumps for PDAs
    /// Bump value
    pub bump: i16,
    /// Treasury bump value
    pub treasury_bump: i16,
    /// Fee payer bump value
    pub fee_payer_bump: i16,

    /// The royalty percentage of the creator, in basis points (0.01%, values
    /// range from 0-10,000)
    pub seller_fee_basis_points: i16,
    /// Boolean value indicating whether the auction house must sign all sales orders.
    pub requires_sign_off: bool,
    /// Whether the Auction House can change the sale price
    ///
    /// Allows the Auction house to do complicated order matching to find the best price for the seller.
    /// Helpful if buyer lists an NFT with price of 0
    pub can_change_sale_price: bool,

    /// Auction House fee account address
    pub auction_house_fee_account: Cow<'a, str>,
}

/// A row in the `bid_reciepts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct BidReceipt<'a> {
    /// The BidReceipt account pubkey
    pub address: Cow<'a, str>,
    /// Trade State account pubkey
    pub trade_state: Cow<'a, str>,
    /// Bookkeeper account pubkey
    pub bookkeeper: Cow<'a, str>,
    /// Auction house account pubkey
    pub auction_house: Cow<'a, str>,
    /// Buyer address
    pub buyer: Cow<'a, str>,
    /// Metadata address
    pub metadata: Cow<'a, str>,
    /// Token account address
    pub token_account: Option<Cow<'a, str>>,
    /// Purchase receipt address
    pub purchase_receipt: Option<Cow<'a, str>>,
    /// Price
    pub price: i64,
    /// Token size
    pub token_size: i64,
    /// Bump
    pub bump: i16,
    /// Trade State bump
    pub trade_state_bump: i16,
    /// Created_at timestamp
    pub created_at: NaiveDateTime,
    /// Canceled_at timestamp
    pub canceled_at: Option<NaiveDateTime>,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: i64,
}

/// A row in the `listing_receipts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(treat_none_as_null = true)]
#[table_name = "listing_receipts"]
pub struct ListingReceipt<'a> {
    /// ListingReceipt account pubkey
    pub address: Cow<'a, str>,
    /// Trade state account pubkey
    pub trade_state: Cow<'a, str>,
    /// Bookkeeper account pubkey
    pub bookkeeper: Cow<'a, str>,
    /// Auction House pubkey
    pub auction_house: Cow<'a, str>,
    /// Seller account pubkey
    pub seller: Cow<'a, str>,
    /// Metadata Address
    pub metadata: Cow<'a, str>,
    /// PurchaseReceipt account address
    pub purchase_receipt: Option<Cow<'a, str>>,
    /// Price
    pub price: i64,
    /// Token Size
    pub token_size: i64,
    /// Bump
    pub bump: i16,
    /// Trade State Bump
    pub trade_state_bump: i16,
    /// Created_at timestamp
    pub created_at: NaiveDateTime,
    /// Canceled_at timestamp
    pub canceled_at: Option<NaiveDateTime>,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: i64,
}

/// A row in the `purchase_receipts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct PurchaseReceipt<'a> {
    /// Purchase account pubkey
    pub address: Cow<'a, str>,
    /// Bookkeeper account pubkey
    pub bookkeeper: Cow<'a, str>,
    /// Buyer account pubkey
    pub buyer: Cow<'a, str>,
    /// Seller account pubkey
    pub seller: Cow<'a, str>,
    /// Auction House account pubkey
    pub auction_house: Cow<'a, str>,
    /// Metadata
    pub metadata: Cow<'a, str>,
    /// Token size
    pub token_size: i64,
    /// Price
    pub price: i64,
    /// Bump
    pub bump: i16,
    /// Created at
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: i64,
}

/// A row in the `store_creators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(treat_none_as_null = true)]
#[table_name = "store_creators"]
pub struct StoreCreator<'a> {
    /// Store Config account address
    pub store_config_address: Cow<'a, str>,
    /// Creator address
    pub creator_address: Cow<'a, str>,
}

/// A row in the `graph_connections` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct GraphConnection<'a> {
    /// Graph Program account address
    pub address: Cow<'a, str>,
    /// Graph Connection 'from' account address
    pub from_account: Cow<'a, str>,
    /// Graph Connection 'to' account address
    pub to_account: Cow<'a, str>,
    /// Graph Connection 'connected_at'
    pub connected_at: NaiveDateTime,
    /// Graph Connection 'disconnected_at'
    pub disconnected_at: Option<NaiveDateTime>,
    /// The slot number of the most recent update for this account
    pub slot: i64,
    /// The write version of the most recent update for this account
    pub write_version: i64,
}

/// A row in the `candy_machines` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct CandyMachine<'a> {
    /// CandyMachine account address
    pub address: Cow<'a, str>,
    /// CandyMachine 'Authority' address
    pub authority: Cow<'a, str>,
    /// CandyMachine 'Wallet' address
    pub wallet: Cow<'a, str>,
    /// Token mint address
    pub token_mint: Option<Cow<'a, str>>,
    /// Items redeemed
    pub items_redeemed: i64,
}

/// A row in the `candy_machine_datas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct CandyMachineData<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Uuid
    pub uuid: Cow<'a, str>,
    /// The amount in SOL or SPL token for a mint
    pub price: i64,
    /// Symbol
    pub symbol: Cow<'a, str>,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: i16,
    /// Max supply
    pub max_supply: i64,
    /// Whether or not the data struct is mutable, default is not
    pub is_mutable: bool,
    /// Indicates whether the candy machine authority has the update authority for each mint
    /// or if it is transferred to the minter
    pub retain_authority: bool,
    /// Timestamp when minting is allowed
    /// the Candy Machine authority and whitelists can bypass this constraint
    pub go_live_date: Option<i64>,
    /// Number of items available
    pub items_available: i64,
}

/// A row in the `candy_machine_config_lines` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_config_lines"]
pub struct CMConfigLine<'a> {
    /// ConfigLine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Name
    pub name: Cow<'a, str>,
    /// URI pointing to JSON representing the asset
    pub uri: Cow<'a, str>,
    /// The index of the config line within the candy machine data
    pub idx: i32,
    /// Bool indicating if this config line has been minted (true) or not minted (false)
    pub taken: bool,
}

/// A row in the `candy_machine_creators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_creators"]
pub struct CMCreator<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Creator account address
    pub creator_address: Cow<'a, str>,
    /// Boolean value to indidicate wheter creator is verified or not
    pub verified: bool,
    /// In percentages, NOT basis points
    pub share: i16,
}

/// A row in the `candy_machine_collection_pdas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_collection_pdas"]
pub struct CMCollectionPDA<'a> {
    /// CollectionPDA address
    pub address: Cow<'a, str>,
    /// Mint address
    pub mint: Cow<'a, str>,
    /// CandyMachine account address
    pub candy_machine: Cow<'a, str>,
}

/// A row in the `candy_machine_hidden_settings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_hidden_settings"]
pub struct CMHiddenSetting<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Name of the mint.
    /// The number of the mint will be appended to the name
    pub name: Cow<'a, str>,
    /// Single URI to all mints
    pub uri: Cow<'a, str>,
    /// 32 character hash
    /// in most cases this is the hash of the cache file with the mapping between
    /// mint number and metadata so that the order can be verified when the mint is complete
    pub hash: Vec<u8>,
}

/// A row in the `candy_machine_whitelist_mint_settings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_whitelist_mint_settings"]
pub struct CMWhitelistMintSetting<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Mode
    /// 'burnEveryTime': true Whitelist token is burned after the mint
    /// 'neverBurn': true Whitelist token is returned to holder
    pub mode: WhitelistMintMode,
    /// Mint address of the whitelist token
    pub mint: Cow<'a, str>,
    /// Indicates whether whitelist token holders can mint before goLiveDate
    pub presale: bool,
    /// Price for whitelist token holders
    pub discount_price: Option<i64>,
}

/// A row in the `candy_machine_gate_keeper_configs` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_gate_keeper_configs"]
pub struct CMGateKeeperConfig<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// Gateway provider address
    pub gatekeeper_network: Cow<'a, str>,
    /// Requires a new gateway challenge after a use
    pub expire_on_use: bool,
}

/// A row in the `candy_machine_end_settings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "candy_machine_end_settings"]
pub struct CMEndSetting<'a> {
    /// CandyMachine account address
    pub candy_machine_address: Cow<'a, str>,
    /// EndSettingType
    /// date : Enable the use of a date to stop the mint
    /// when the date specified in the value option is reached, the mint stops
    /// amount: Enable stopping the mint after a specific amount is minted
    /// the amount is specified in the value option
    pub end_setting_type: EndSettingType,
    /// Value to test the end condition.
    /// This will be either a date (if date is set to true)
    /// or a integer amount value (if amount is set to true)
    pub number: i64,
}

/// A row in a `mint_stats` query, representing stats for a single token type
/// identified by its mint
#[derive(Debug, Clone, QueryableByName)]
pub struct MintStats<'a> {
    /// The auction house for which stats were collected
    #[sql_type = "VarChar"]
    pub auction_house: Cow<'a, str>,
    /// The mint of this token
    #[sql_type = "Text"]
    pub mint: Cow<'a, str>,
    /// The floor price in this token
    #[sql_type = "Nullable<Int8>"]
    pub floor: Option<i64>,
    /// The average price in this token
    #[sql_type = "Nullable<Int8>"]
    pub average: Option<i64>,
    /// 24-hour volume for this token
    #[sql_type = "Nullable<Int8>"]
    pub volume_24hr: Option<i64>,
    /// Total volume for this token
    #[sql_type = "Nullable<Int8>"]
    pub volume_total: Option<i64>,
}

/// A join of `graph_connections` and `twitter_handle_name_services` for connections that include twitter handle of wallets
#[derive(Debug, Clone, QueryableByName)]
pub struct TwitterEnrichedGraphConnection {
    /// The address of the connection
    #[sql_type = "VarChar"]
    pub connection_address: String,
    /// The from_account of the connection
    #[sql_type = "VarChar"]
    pub from_account: String,
    /// The to_account of the connection
    #[sql_type = "VarChar"]
    pub to_account: String,
    /// Graph Connection 'connected_at'
    #[sql_type = "Timestamp"]
    pub connected_at: NaiveDateTime,
    /// Graph Connection 'disconnected_at'
    #[sql_type = "Nullable<Timestamp>"]
    pub disconnected_at: Option<NaiveDateTime>,
    /// The twitter handle of the from_account
    #[sql_type = "Nullable<Text>"]
    pub from_twitter_handle: Option<String>,
    /// The twitter handle of the to_account
    #[sql_type = "Nullable<Text>"]
    pub to_twitter_handle: Option<String>,
}

/// A row in a `charts` query, representing requested price data on a particualar date
#[derive(Debug, Clone, Copy, QueryableByName)]
pub struct PricePoint {
    /// The requested price on a date
    #[sql_type = "Int8"]
    pub price: i64,

    /// The date for which the price was requested
    #[sql_type = "Timestamp"]
    pub date: NaiveDateTime,
}

/// A row in a `metadatas::count_by_marketplace` query, representing stats for
/// a single marketplace
#[derive(Debug, Clone, QueryableByName)]
pub struct MarketStats<'a> {
    /// The store config address of the marketplace for which stats were
    /// collected
    #[sql_type = "VarChar"]
    pub store_config: Cow<'a, str>,
    /// Number of NFTs in this marketplace
    #[sql_type = "Nullable<Int8>"]
    pub nfts: Option<i64>,
}

/// A row in the `twitter_handle_name_services` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "twitter_handle_name_services"]
pub struct TwitterHandle<'a> {
    /// NameService account address
    pub address: Cow<'a, str>,
    /// Wallet address of twitter handle owner
    pub wallet_address: Cow<'a, str>,
    /// Twitter handle
    pub twitter_handle: Cow<'a, str>,
    /// Solana slot number
    pub slot: i64,
    /// from bonfida
    pub from_bonfida: bool,
    /// from cardinal
    pub from_cardinal: bool,
    /// write version from solana
    pub write_version: i64,
}

/// A row in a `collected_collections` query of a wallet
#[derive(Debug, Clone, QueryableByName)]
pub struct CollectedCollection {
    /// The collection nft metadadata address
    #[sql_type = "VarChar"]
    pub collection_nft_address: String,
    /// The nfts from this collection owned by the wallet
    #[sql_type = "Int8"]
    pub nfts_owned: i64,
    /// The estimated value of the collection owend by the wallet
    #[sql_type = "Int8"]
    pub estimated_value: i64,
}

/// A row in a `created_collections` query of a wallet
#[derive(Debug, Clone, QueryableByName)]
pub struct CreatedCollection {
    /// The metadata address for the collection
    #[sql_type = "VarChar"]
    pub address: String,
}

/// A row in the `metadata_collection_keys` table
/// Each collection is an NFT
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct MetadataCollectionKey<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// Collection Address
    pub collection_address: Cow<'a, str>,
    /// Whether the collection is verified or not.
    pub verified: bool,
}

/// Joint table from querying a `token_manager` and related plugins
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct CardinalTokenManagerQuery {
    /// Address of the token_manager
    #[sql_type = "Text"]
    pub address: String,
    /// Version of the token_manager
    #[sql_type = "Int4"]
    pub version: i16,
    /// Bump seed of the token_manager
    #[sql_type = "Int4"]
    pub bump: i16,
    /// Count for the given mint to identify this token_manager
    #[sql_type = "Int8"]
    pub count: i64,
    /// Max number of invalidators this token_manager can hold
    #[sql_type = "Int4"]
    pub num_invalidators: i16,
    /// Issuer of this token_manager
    #[sql_type = "Text"]
    pub issuer: String,
    /// The mint that this token_manager holder
    #[sql_type = "Text"]
    pub mint: String,
    /// How many of the given mint in this token_manager
    #[sql_type = "Int8"]
    pub amount: i64,
    /// Kind of this token_manager
    #[sql_type = "Int4"]
    pub kind: i16,
    /// Current state of the token_manager
    #[sql_type = "Int4"]
    pub state: i16,
    /// Timestamp in seconds for last state change
    #[sql_type = "Timestamp"]
    pub state_changed_at: NaiveDateTime,
    /// What happens upon invalidation
    #[sql_type = "Int4"]
    pub invalidation_type: i16,
    /// Current token_account holding this managed token
    #[sql_type = "Text"]
    pub recipient_token_account: String,
    /// Optional receipt claimed from the token_manager representing the rightful owner
    #[sql_type = "Nullable<Text>"]
    pub receipt_mint: Option<String>,
    /// Option authority that can approve claiming the token
    #[sql_type = "Nullable<Text>"]
    pub claim_approver: Option<String>,
    /// Optional authority that can approve transfers (defaults to self)
    #[sql_type = "Nullable<Text>"]
    pub transfer_authority: Option<String>,
    /// Amount the pay for extension
    #[sql_type = "Nullable<Int8>"]
    pub paid_claim_approver_payment_amount: Option<i64>,
    /// Mint that extension is denominated in
    #[sql_type = "Nullable<Text>"]
    pub paid_claim_approver_payment_mint: Option<String>,
    /// payment manager
    #[sql_type = "Nullable<Text>"]
    pub paid_claim_approver_payment_manager: Option<String>,
    /// collector
    #[sql_type = "Nullable<Text>"]
    pub paid_claim_approver_collector: Option<String>,
    /// address
    #[sql_type = "Nullable<Text>"]
    pub time_invalidator_address: Option<String>,
    /// payment manager
    #[sql_type = "Nullable<Text>"]
    pub time_invalidator_payment_manager: Option<String>,
    /// collector
    #[sql_type = "Nullable<Text>"]
    pub time_invalidator_collector: Option<String>,
    /// Optional expiration which this time invalidator will expire
    #[sql_type = "Nullable<Int8>"]
    pub time_invalidator_expiration: Option<NaiveDateTime>,
    /// Duration after claim
    #[sql_type = "Nullable<Int8>"]
    pub time_invalidator_duration_seconds: Option<i64>,
    /// Amount the pay for extension
    #[sql_type = "Nullable<Int8>"]
    pub time_invalidator_extension_payment_amount: Option<i64>,
    /// Duration received after extension
    #[sql_type = "Nullable<Int8>"]
    pub time_invalidator_extension_duration_seconds: Option<i64>,
    /// Mint that extension is denominated in
    #[sql_type = "Nullable<Text>"]
    pub time_invalidator_extension_payment_mint: Option<String>,
    /// Optional max this can ever be extended until
    #[sql_type = "Nullable<Timestamp>"]
    pub time_invalidator_max_expiration: Option<NaiveDateTime>,
    /// Whether extension can be in partial increments
    #[sql_type = "Nullable<Bool>"]
    pub time_invalidator_disable_partial_extension: Option<bool>,
    /// address
    #[sql_type = "Nullable<Text>"]
    pub use_invalidator_address: Option<String>,
    /// use invalidator payment manager
    #[sql_type = "Nullable<Text>"]
    pub use_invalidator_payment_manager: Option<String>,
    /// use_invalidator_collector
    #[sql_type = "Nullable<Text>"]
    pub use_invalidator_collector: Option<String>,
    /// use_invalidator_usages
    #[sql_type = "Nullable<Int8>"]
    pub use_invalidator_usages: Option<i64>,
    /// use_invalidator_use_authority
    #[sql_type = "Nullable<Text>"]
    pub use_invalidator_use_authority: Option<String>,
    /// use_invalidator_total_usages
    #[sql_type = "Nullable<Int8>"]
    pub use_invalidator_total_usages: Option<i64>,
    /// use_invalidator_extension_payment_amount
    #[sql_type = "Nullable<Int8>"]
    pub use_invalidator_extension_payment_amount: Option<i64>,
    /// use_invalidator_extension_payment_mint
    #[sql_type = "Nullable<Text>"]
    pub use_invalidator_extension_payment_mint: Option<String>,
    /// use_invalidator_extension_usages
    #[sql_type = "Nullable<Int8>"]
    pub use_invalidator_extension_usages: Option<i64>,
    /// use_invalidator_max_usages
    #[sql_type = "Nullable<Int8>"]
    pub use_invalidator_max_usages: Option<i64>,
}

/// A row in the `cardinal_token_managers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_token_managers"]
pub struct CardinalTokenManager<'a> {
    /// Address of the token_manager
    pub address: Cow<'a, str>,
    /// Version of the token_manager
    pub version: i16,
    /// Bump seed of the token_manager
    pub bump: i16,
    /// Count for the given mint to identify this token_manager
    pub count: i64,
    /// Max number of invalidators this token_manager can hold
    pub num_invalidators: i16,
    /// Issuer of this token_manager
    pub issuer: Cow<'a, str>,
    /// The mint that this token_manager holder
    pub mint: Cow<'a, str>,
    /// How many of the given mint in this token_manager
    pub amount: i64,
    /// Kind of this token_manager
    pub kind: i16,
    /// Current state of the token_manager
    pub state: i16,
    /// Timestamp in seconds for last state change
    pub state_changed_at: NaiveDateTime,
    /// What happens upon invalidation
    pub invalidation_type: i16,
    /// Current token_account holding this managed token
    pub recipient_token_account: Cow<'a, str>,
    /// Optional receipt claimed from the token_manager representing the rightful owner
    pub receipt_mint: Option<Cow<'a, str>>,
    /// Option authority that can approve claiming the token
    pub claim_approver: Option<Cow<'a, str>>,
    /// Optional authority that can approve transfers (defaults to self)
    pub transfer_authority: Option<Cow<'a, str>>,
}

/// A row in the `cardinal_token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_token_manager_invalidators"]
pub struct CardinalTokenManagerInvalidator<'a> {
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Address of an active invalidator for this token_manager
    pub invalidator: Cow<'a, str>,
}

/// A row in the `cardinal_time_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_time_invalidators"]
pub struct CardinalTimeInvalidator<'a> {
    /// Address of the time_invalidator
    pub time_invalidator_address: Cow<'a, str>,
    /// Bump seed of the time_invalidator
    pub time_invalidator_bump: i16,
    /// Address of the token_manager
    pub time_invalidator_token_manager_address: Cow<'a, str>,
    /// Address of the payment manager
    pub time_invalidator_payment_manager: Cow<'a, str>,
    /// Address of the collector
    pub time_invalidator_collector: Cow<'a, str>,
    /// Optional expiration which this time invalidator will expire
    pub time_invalidator_expiration: Option<NaiveDateTime>,
    /// Duration after claim
    pub time_invalidator_duration_seconds: Option<i64>,
    /// Amount the pay for extension
    pub time_invalidator_extension_payment_amount: Option<i64>,
    /// Duration received after extension
    pub time_invalidator_extension_duration_seconds: Option<i64>,
    /// Mint that extension is denominated in
    pub time_invalidator_extension_payment_mint: Option<Cow<'a, str>>,
    /// Optional max this can ever be extended until
    pub time_invalidator_max_expiration: Option<NaiveDateTime>,
    /// Whether extension can be in partial increments
    pub time_invalidator_disable_partial_extension: Option<bool>,
}

/// A row in the `cardinal_use_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_use_invalidators"]
pub struct CardinalUseInvalidator<'a> {
    /// Address of the use_invalidator
    pub use_invalidator_address: Cow<'a, str>,
    /// Bump seed of the use_invalidator
    pub use_invalidator_bump: i16,
    /// Address of the token_manager
    pub use_invalidator_token_manager_address: Cow<'a, str>,
    /// Address of the payment manager
    pub use_invalidator_payment_manager: Cow<'a, str>,
    /// Address of the collector
    pub use_invalidator_collector: Cow<'a, str>,
    /// Optional expiration which this time invalidator will expire
    pub use_invalidator_usages: i64,
    /// Address that can increment usages
    pub use_invalidator_use_authority: Option<Cow<'a, str>>,
    /// Total usages
    pub use_invalidator_total_usages: Option<i64>,
    /// Amount the pay for extension
    pub use_invalidator_extension_payment_amount: Option<i64>,
    /// Mint that extension is denominated in
    pub use_invalidator_extension_payment_mint: Option<Cow<'a, str>>,
    /// Number of usages received after extension
    pub use_invalidator_extension_usages: Option<i64>,
    /// Optional max this can ever be extended until
    pub use_invalidator_max_usages: Option<i64>,
}

/// A row in the `cardinal_token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_paid_claim_approvers"]
pub struct CardinalPaidClaimApprover<'a> {
    /// Address of the use_invalidator
    pub paid_claim_approver_address: Cow<'a, str>,
    /// Bump seed of the use_invalidator
    pub paid_claim_approver_bump: i16,
    /// Address of the token_manager
    pub paid_claim_approver_token_manager_address: Cow<'a, str>,
    /// Address of the payment manager
    pub paid_claim_approver_payment_manager: Cow<'a, str>,
    /// Address of the collector
    pub paid_claim_approver_collector: Cow<'a, str>,
    /// Amount the pay for extension
    pub paid_claim_approver_payment_amount: i64,
    /// Mint that extension is denominated in
    pub paid_claim_approver_payment_mint: Cow<'a, str>,
}

/// A row in the `cardinal_claim_events` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_claim_events"]
pub struct CardinalClaimEvent<'a> {
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Version of the token_manager
    pub version: i16,
    /// Bump seed of the token_manager
    pub bump: i16,
    /// Count for the given mint to identify this token_manager
    pub count: i64,
    /// Max number of invalidators this token_manager can hold
    pub num_invalidators: i16,
    /// Issuer of this token_manager
    pub issuer: Cow<'a, str>,
    /// The mint that this token_manager holder
    pub mint: Cow<'a, str>,
    /// How many of the given mint in this token_manager
    pub amount: i64,
    /// Kind of this token_manager
    pub kind: i16,
    /// Current state of the token_manager
    pub state: i16,
    /// Timestamp in seconds for last state change
    pub state_changed_at: NaiveDateTime,
    /// What happens upon invalidation
    pub invalidation_type: i16,
    /// Current token_account holding this managed token
    pub recipient_token_account: Cow<'a, str>,
    /// Optional receipt claimed from the token_manager representing the rightful owner
    pub receipt_mint: Option<Cow<'a, str>>,
    /// Option authority that can approve claiming the token
    pub claim_approver: Option<Cow<'a, str>>,
    /// Optional authority that can approve transfers (defaults to self)
    pub transfer_authority: Option<Cow<'a, str>>,
    // /// Listof invalidators
    // pub invalidators: Vec<Cow<'a, str>>,
    /// Amount the pay for extension
    pub paid_claim_approver_payment_amount: Option<i64>,
    /// Mint that extension is denominated in
    pub paid_claim_approver_payment_mint: Option<Cow<'a, str>>,
    /// Payment manager address
    pub paid_claim_approver_payment_manager: Option<Cow<'a, str>>,
    /// Claim approver collector
    pub paid_claim_approver_collector: Option<Cow<'a, str>>,
    /// Time invalidator address
    pub time_invalidator_address: Option<Cow<'a, str>>,
    /// Time inavlidator payment manager address
    pub time_invalidator_payment_manager: Option<Cow<'a, str>>,
    /// Time inavlidator collector
    pub time_invalidator_collector: Option<Cow<'a, str>>,
    /// Optional expiration which this time invalidator will expire
    pub time_invalidator_expiration: Option<NaiveDateTime>,
    /// Duration after claim
    pub time_invalidator_duration_seconds: Option<i64>,
    /// Amount the pay for extension
    pub time_invalidator_extension_payment_amount: Option<i64>,
    /// Duration received after extension
    pub time_invalidator_extension_duration_seconds: Option<i64>,
    /// Mint that extension is denominated in
    pub time_invalidator_extension_payment_mint: Option<Cow<'a, str>>,
    /// Optional max this can ever be extended until
    pub time_invalidator_max_expiration: Option<NaiveDateTime>,
    /// Whether extension can be in partial increments
    pub time_invalidator_disable_partial_extension: Option<bool>,
    /// Use invalidator address
    pub use_invalidator_address: Option<Cow<'a, str>>,
    /// Use inavlidator payment manager address
    pub use_invalidator_payment_manager: Option<Cow<'a, str>>,
    /// Use inavlidator collector
    pub use_invalidator_collector: Option<Cow<'a, str>>,
    /// Optional expiration which this time invalidator will expire
    pub use_invalidator_usages: Option<i64>,
    /// Address that can increment usages
    pub use_invalidator_use_authority: Option<Cow<'a, str>>,
    /// Total usages
    pub use_invalidator_total_usages: Option<i64>,
    /// Amount the pay for extension
    pub use_invalidator_extension_payment_amount: Option<i64>,
    /// Mint that extension is denominated in
    pub use_invalidator_extension_payment_mint: Option<Cow<'a, str>>,
    /// Number of usages received after extension
    pub use_invalidator_extension_usages: Option<i64>,
    /// Optional max this can ever be extended until
    pub use_invalidator_max_usages: Option<i64>,
}

/// `Tribeca` Locked-Voter program account
/// A row in the `lockers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Locker<'a> {
    /// `Locker` account pubkey
    pub address: Cow<'a, str>,
    /// Base account used to generate signer seeds.
    pub base: Cow<'a, str>,
    /// Bump seed
    pub bump: i16,
    /// Mint of the token that must be locked in the [Locker].
    pub token_mint: Cow<'a, str>,
    /// Total number of tokens locked in [Escrow]s.
    pub locked_supply: i64,
    /// Governor associated with the [Locker].
    pub governor: Cow<'a, str>,
}

/// A row in the `locker_params` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct LockerParam<'a> {
    /// `Locker` account pubkey
    pub locker_address: Cow<'a, str>,
    /// Whether or not the locking whitelist system is enabled.
    pub whitelist_enabled: bool,
    /// The weight of a maximum vote lock relative to the total number of tokens locked.
    pub max_stake_vote_multiplier: i16,
    /// Minimum staking duration.
    pub min_stake_duration: i64,
    /// Maximum staking duration.
    pub max_stake_duration: i64,
    /// Minimum number of votes required to activate a proposal.
    pub proposal_activation_min_votes: i64,
}

/// `Tribeca` Locked-Voter program account
/// A row in the `locker_whitelist_entries` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "locker_whitelist_entries"]
pub struct LockerWhitelistEntry<'a> {
    /// `LockerWhitelistEntry` account pubkey
    pub address: Cow<'a, str>,
    /// Bump seed.
    pub bump: i16,
    /// [Locker] this whitelist entry belongs to.
    pub locker: Cow<'a, str>,
    /// Key of the program_id allowed to call the `lock` CPI.
    pub program_id: Cow<'a, str>,
    /// The account authorized to be the [Escrow::owner] with this CPI.
    pub owner: Cow<'a, str>,
}

/// `Tribeca` Locked-Voter program account
/// A row in the `escrows` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Escrow<'a> {
    /// `Escrow` account pubkey
    pub address: Cow<'a, str>,
    /// The [Locker] that this [Escrow] is part of.
    pub locker: Cow<'a, str>,
    /// The key of the account that is authorized to stake into/withdraw from this [Escrow].
    pub owner: Cow<'a, str>,
    /// Bump seed.
    pub bump: i16,
    /// The token account holding the escrow tokens.
    pub tokens: Cow<'a, str>,
    /// Amount of tokens staked.
    pub amount: i64,
    /// When the [Escrow::owner] started their escrow.
    pub escrow_started_at: i64,
    /// When the escrow unlocks; i.e. the [Escrow::owner] is scheduled to be allowed to withdraw their tokens.
    pub escrow_ends_at: i64,

    /// Account that is authorized to vote on behalf of this [Escrow].
    /// Defaults to the [Escrow::owner].
    pub vote_delegate: Cow<'a, str>,
}

/// `Tribeca` Govern program account
/// A row in the `governors` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Governor<'a> {
    /// `Governor` account pubkey
    pub address: Cow<'a, str>,
    /// Base.
    pub base: Cow<'a, str>,
    /// Bump seed
    pub bump: i16,
    /// The total number of Proposals
    pub proposal_count: i64,

    /// The voting body associated with the Governor.
    /// This account is responsible for handling vote proceedings, such as:
    /// - activating proposals
    /// - setting the number of votes per voter
    pub electorate: Cow<'a, str>,
    /// The public key of the `smart_wallet::SmartWallet` account.
    /// This smart wallet executes proposals.
    pub smart_wallet: Cow<'a, str>,
}

/// A row in the `governor_parameters` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct GovernanceParameter<'a> {
    /// `Governor` account pubkey
    pub governor_address: Cow<'a, str>,
    /// The delay before voting on a proposal may take place, once proposed, in seconds
    pub voting_delay: i64,
    /// The duration of voting on a proposal, in seconds
    pub voting_period: i64,
    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: i64,
    /// The timelock delay of the DAO's created proposals.
    pub timelock_delay_seconds: i64,
}

/// `Tribeca` Govern program account
/// A row in the `proposals` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Proposal<'a> {
    /// Proposal account pubkey
    pub address: Cow<'a, str>,
    /// The public key of the governor.
    pub governor: Cow<'a, str>,
    /// The unique ID of the proposal, auto-incremented.
    pub index: i64,
    /// Bump seed
    pub bump: i16,
    /// The public key of the proposer.
    pub proposer: Cow<'a, str>,
    /// The number of votes in support of a proposal required in order for a quorum to be reached and for a vote to succeed
    pub quorum_votes: i64,
    /// Current number of votes in favor of this proposal
    pub for_votes: i64,
    /// Current number of votes in opposition to this proposal
    pub against_votes: i64,
    /// Current number of votes for abstaining for this proposal
    pub abstain_votes: i64,
    /// The timestamp when the proposal was canceled.
    pub canceled_at: i64,
    /// The timestamp when the proposal was created.
    pub created_at: i64,
    /// The timestamp in which the proposal was activated.
    /// This is when voting begins.
    pub activated_at: i64,
    /// The timestamp when voting ends.
    /// This only applies to active proposals.
    pub voting_ends_at: i64,
    /// The timestamp in which the proposal was queued, i.e.
    /// approved for execution on the Smart Wallet.
    pub queued_at: i64,
    /// If the transaction was queued, this is the associated Goki Smart Wallet transaction.
    pub queued_transaction: Cow<'a, str>,
}

/// A row in the `proposal_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct ProposalInstruction<'a> {
    /// public key of the proposal to which the instruction is associated
    pub proposal_address: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Cow<'a, str>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

/// A row in the `proposal_account_metas` table
/// Account metadata used to define Instructions
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct ProposalAccountMeta<'a> {
    /// Pubkey of the proposal to which the account metadata is associated
    pub proposal_address: Cow<'a, str>,
    /// Pubkey of the program id which executes the instruction to which the account metadata is associated
    pub program_id: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes the instruction to which the account metadata is associated
    pub pubkey: Cow<'a, str>,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}

/// `Tribeca` Govern program account
/// A row in the `proposal_metas` table
/// Metadata about a proposal.
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct ProposalMeta<'a> {
    /// `ProposalMeta` account pubkey
    pub address: Cow<'a, str>,
    /// Pubkey of the proposal to which metadata is associated
    pub proposal: Cow<'a, str>,
    /// Title of the proposal.
    pub title: Cow<'a, str>,
    /// Link to a description of the proposal.
    pub description_link: Cow<'a, str>,
}

/// `Tribeca` Govern program account
/// A row in the `votes` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Vote<'a> {
    /// `Vote` account pubkey
    pub address: Cow<'a, str>,
    /// Pubkey of the proposal being voted on.
    pub proposal: Cow<'a, str>,
    /// Pubkey of the voter
    pub voter: Cow<'a, str>,
    /// Bump seed
    pub bump: i16,
    /// The side of the vote taken.
    pub side: i16,
    /// The number of votes this vote holds.
    pub weight: i64,
}

/// A row in the `smart_wallets` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct SmartWallet<'a> {
    /// Smart Wallet account pubkey
    pub address: Cow<'a, str>,
    /// Base used to derive.
    pub base: Cow<'a, str>,
    /// Bump seed for deriving PDA seeds.
    pub bump: i16,
    /// Minimum number of owner approvals needed to sign a [Transaction].
    pub threshold: i64,
    /// Minimum delay between approval and execution, in seconds.
    pub minimum_delay: i64,
    /// Time after the ETA until a [Transaction] expires.
    pub grace_period: i64,
    ///Sequence of the ownership set.
    pub owner_set_seqno: i64,
    /// Total number of [Transaction]s on this [SmartWallet].
    pub num_transactions: i64,
}

/// A row in the `smart_wallet_owners` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct SmartWalletOwner<'a> {
    /// Smart Wallet account pubkey
    pub smart_wallet_address: Cow<'a, str>,
    /// Owners of the [SmartWallet].
    pub owner_address: Cow<'a, str>,
    /// Position of owner in vec<Owners Pubkey>
    pub index: i64,
}

/// A row in the `transactions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Transaction<'a> {
    /// Transaction account pubkey
    pub address: Cow<'a, str>,
    /// The [SmartWallet] account this transaction belongs to.
    pub smart_wallet: Cow<'a, str>,
    /// The auto-incremented integer index of the transaction.
    /// All transactions on the [SmartWallet] can be looked up via this index,
    /// allowing for easier browsing of a wallet's historical transactions.
    pub index: i64,
    /// Bump seed.
    pub bump: i16,
    /// The proposer of the [Transaction].
    pub proposer: Cow<'a, str>,
    /// `signers[index]` is true iff `[SmartWallet]::owners[index]` signed the transaction.
    pub signers: Vec<bool>,
    /// Owner set sequence number.
    pub owner_set_seqno: i64,
    /// Estimated time the [Transaction] will be executed.
    pub eta: i64,
    /// The account that executed the [Transaction].
    pub executor: Cow<'a, str>,
    /// When the transaction was executed. -1 if not executed.
    pub executed_at: i64,
}

/// A row in the `tx_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "tx_instructions"]
pub struct TXInstruction<'a> {
    /// Transaction account pubkey
    pub transaction_address: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Cow<'a, str>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

/// A row in the `tx_instruction_keys` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "tx_instruction_keys"]
pub struct TXInstructionKey<'a> {
    /// Transaction account pubkey
    pub transaction_address: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Cow<'a, str>,
    /// An account's public key
    pub pubkey: Cow<'a, str>,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}

/// A row in the `subaccount_infos` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct SubAccountInfo<'a> {
    /// SubAccountInfo account pubkey
    pub address: Cow<'a, str>,
    /// Smart wallet of the sub-account.
    pub smart_wallet: Cow<'a, str>,
    /// Type of sub-account.
    /// 0 -> Requires the normal multisig approval process.
    /// 1 ->Any owner may sign an instruction  as this address.
    pub subaccount_type: i16,
    /// Index of the sub-account.
    pub index: i64,
}

/// A row in the `instruction_buffers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct InstructionBuffer<'a> {
    /// InstructionBuffer account pubkey
    pub address: Cow<'a, str>,
    /// Sequence of the ownership set.
    pub owner_set_seqno: i64,
    /// - If set to `NO_ETA`, the instructions in each `InstructionBuffer::bundles` may be executed at any time.
    /// - Otherwise, instructions may be executed at any point after the ETA has elapsed.
    pub eta: i64,
    /// Authority of the buffer.
    pub authority: Cow<'a, str>,
    /// Role that can execute instructions off the buffer.
    pub executor: Cow<'a, str>,
    /// Smart wallet the buffer belongs to.
    pub smart_wallet: Cow<'a, str>,
}

/// A row in the `ins_buffer_bundles` table
/// Vector of instructions.
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct InsBufferBundle<'a> {
    /// InstructionBuffer account pubkey
    pub instruction_buffer_address: Cow<'a, str>,
    /// Execution counter on the `InstructionBundle`.
    pub is_executed: bool,
}

/// A row in the `ins_buffer_bundle_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "ins_buffer_bundle_instructions"]
pub struct InsBuffferBundleInstruction<'a> {
    /// InstructionBuffer account pubkey
    pub instruction_buffer_address: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Cow<'a, str>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

/// A row in the `ins_buffer_bundle_ins_keys` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct InsBufferBundleInsKey<'a> {
    /// InstructionBuffer account pubkey
    pub instruction_buffer_address: Cow<'a, str>,
    /// Pubkey of the instruction processor that executes the instruction
    pub program_id: Cow<'a, str>,
    /// An account's public key
    pub pubkey: Cow<'a, str>,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}
/// A row in the `bonding_change` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "bonding_changes"]
pub struct BondingChange<'a> {
    /// Account address
    pub address: Cow<'a, str>,
    /// Insert timestamp
    pub insert_ts: NaiveDateTime,
    /// The solana slot
    pub slot: i64,
    /// Current value of the reserves_from_bonding field
    pub current_reserves_from_bonding: i64,
    /// Current value of the supply_from_bonding field
    pub current_supply_from_bonding: i64,
}

/// An enriched query on bonding changes
#[derive(Debug, Clone, QueryableByName)]
#[diesel(treat_none_as_null = true)]
pub struct EnrichedBondingChange<'a> {
    /// Account address
    #[sql_type = "Text"]
    pub address: Cow<'a, str>,
    /// The solana slot
    #[sql_type = "Int8"]
    pub slot: i64,
    /// Insert timestamp
    #[sql_type = "Timestamp"]
    pub insert_ts: NaiveDateTime,
    /// The observed reserve change
    #[sql_type = "Int8"]
    pub reserve_change: i64,
    ///The observed supply change
    #[sql_type = "Int8"]
    pub supply_change: i64,
}

/// A row in the `metadata_owners` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct CurrentMetadataOwner<'a> {
    /// The mint address of the token
    pub mint_address: Cow<'a, str>,
    /// The token owner address
    pub owner_address: Cow<'a, str>,
    /// The address of token account
    pub token_account_address: Cow<'a, str>,
    /// Solana slot number
    /// The period of time for which each leader ingests transactions and produces a block.
    pub slot: i64,
}

/// A row in a `metadatas::count_by_store_creator` query, representing stats for
/// a store creator
#[derive(Debug, Clone, QueryableByName)]
pub struct StoreCreatorCount<'a> {
    /// The store creator's address for which stats were
    /// collected
    #[sql_type = "VarChar"]
    pub store_creator: Cow<'a, str>,
    /// Number of NFTs creatred by this store_creator
    #[sql_type = "Int8"]
    pub nfts: i64,
}

/// A join of all `feed_events` related tables into a complete feed event record
#[derive(Debug, Clone, QueryableByName)]
pub struct CompleteFeedEvent {
    /// generated id for the event
    #[sql_type = "diesel::sql_types::Uuid"]
    pub id: Uuid,
    /// generated created_at
    #[sql_type = "Timestamptz"]
    pub created_at: NaiveDateTime,
    /// wallet associated to the event
    #[sql_type = "VarChar"]
    pub wallet_address: String,
    /// potentially twitter handle for associated wallet
    #[sql_type = "Nullable<Text>"]
    pub twitter_handle: Option<String>,
    /// metadata address that triggered the mint event
    #[sql_type = "Nullable<VarChar>"]
    pub metadata_address: Option<String>,
    /// purchase id that triggered the purchase event
    #[sql_type = "Nullable<diesel::sql_types::Uuid>"]
    pub purchase_id: Option<Uuid>,
    #[sql_type = "Nullable<diesel::sql_types::Uuid>"]
    /// offer id that triggered the offer event
    pub offer_id: Option<Uuid>,
    /// the lifecycle of the offer event
    #[sql_type = "Nullable<OfferEventLifecycle>"]
    pub offer_lifecycle: Option<OfferEventLifecycleEnum>,
    /// listing id that triggered the listing event
    #[sql_type = "Nullable<diesel::sql_types::Uuid>"]
    pub listing_id: Option<Uuid>,
    /// the lifecycle of the listing event
    #[sql_type = "Nullable<ListingEventLifecycle>"]
    pub listing_lifecycle: Option<ListingEventLifecycleEnum>,
    /// graph connection address that triggered the follow event
    #[sql_type = "Nullable<VarChar>"]
    pub graph_connection_address: Option<String>,
}

/// A row in the `feed_events` table
#[derive(Debug, Clone, Copy, Queryable, Insertable)]
#[table_name = "feed_events"]
pub struct FeedEvent {
    /// generated id
    pub id: Uuid,
    /// generated created_at
    pub created_at: NaiveDateTime,
}

/// A row in the `feed_event_wallets` table
#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "feed_event_wallets"]
pub struct FeedEventWallet<'a> {
    /// a wallet associated to the event
    pub wallet_address: Cow<'a, str>,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
}

/// A row in the `mint_events` table
#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "mint_events"]
pub struct MintEvent<'a> {
    /// foreign key to `metadatas` address
    pub metadata_address: Cow<'a, str>,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
}

/// A row in the `offer_events` table
#[derive(Debug, Clone, Copy, Queryable, Insertable)]
#[table_name = "offer_events"]
pub struct OfferEvent {
    /// foreign key to `offers` id
    pub offer_id: Uuid,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
    ///  enum of offer lifecycle
    pub lifecycle: OfferEventLifecycleEnum,
}

/// A row in the `listing_events` table
#[derive(Debug, Clone, Copy, Queryable, Insertable)]
#[table_name = "listing_events"]
pub struct ListingEvent {
    /// foreign key to `listings` id
    pub listing_id: Uuid,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
    /// enum of listing lifecycle
    pub lifecycle: ListingEventLifecycleEnum,
}

/// A row in the `purchase_events` table
#[derive(Debug, Clone, Copy, Queryable, Insertable)]
#[table_name = "purchase_events"]
pub struct PurchaseEvent {
    /// foreign key to `purchases` id
    pub purchase_id: Uuid,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
}

/// A row in the `follow_events` table
#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "follow_events"]
pub struct FollowEvent<'a> {
    /// foreign key to `graph_connections` address
    pub graph_connection_address: Cow<'a, str>,
    /// foreign key to `feed_events`
    pub feed_event_id: Uuid,
}

/// A row in the `wallet_totals` table
#[derive(Debug, Clone, Queryable)]
pub struct WalletTotal {
    /// wallet address
    pub address: String,
    /// wallet follwers
    pub followers: i64,
    /// wallet following
    pub following: i64,
}

/// A row in the `store_auction_houses` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(treat_none_as_null = true)]
#[table_name = "store_auction_houses"]
pub struct StoreAuctionHouse<'a> {
    /// Store Config account address
    pub store_config_address: Cow<'a, str>,
    /// Auction House address
    pub auction_house_address: Cow<'a, str>,
}
/// A row in the `buy_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct BuyInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Wallet used to pay for the Bid
    pub payment_account: Cow<'a, str>,
    /// Transfer authority pubkey
    pub transfer_authority: Cow<'a, str>,
    /// Treasury mint pubkey
    pub treasury_mint: Cow<'a, str>,
    /// Nft Token account pubkey
    pub token_account: Cow<'a, str>,
    /// Metadata account pubkey
    pub metadata: Cow<'a, str>,
    /// Escrow account pubkey where funds are deposited
    pub escrow_payment_account: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Auction house pubkey
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// Buyer trade state account pubkey
    pub buyer_trade_state: Cow<'a, str>,
    /// trade state bump
    pub trade_state_bump: i16,
    /// escrow payment bump
    pub escrow_payment_bump: i16,
    /// buyer price in lamports
    pub buyer_price: i64,
    /// Token size (usually 1)
    pub token_size: i64,
    /// Timestamp when 'Buy' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `public_buy_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct PublicBuyInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Wallet used to pay for the Bid
    pub payment_account: Cow<'a, str>,
    /// Transfer authority pubkey
    pub transfer_authority: Cow<'a, str>,
    /// Treasury mint pubkey
    pub treasury_mint: Cow<'a, str>,
    /// Nft Token account pubkey
    pub token_account: Cow<'a, str>,
    /// Metadata account pubkey
    pub metadata: Cow<'a, str>,
    /// Escrow account pubkey where funds are deposited
    pub escrow_payment_account: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Auction house pubkey
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// Buyer trade state account pubkey
    pub buyer_trade_state: Cow<'a, str>,
    /// trade state bump
    pub trade_state_bump: i16,
    /// escrow payment bump
    pub escrow_payment_bump: i16,
    /// buyer price in lamports
    pub buyer_price: i64,
    /// Token size (usually 1)
    pub token_size: i64,
    /// Timestamp when 'Buy' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `sell_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct SellInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Nft Token account pubkey
    pub token_account: Cow<'a, str>,
    /// Metadata account pubkey
    pub metadata: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Auction house pubkey
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// Seller trade state pubkey
    pub seller_trade_state: Cow<'a, str>,
    /// free_seller_trade_state pubkey
    pub free_seller_trader_state: Cow<'a, str>,
    /// Program address signing the transaction
    pub program_as_signer: Cow<'a, str>,
    /// trade state bump
    pub trade_state_bump: i16,
    /// free trade state bump
    pub free_trade_state_bump: i16,
    /// program_as_signer bump
    pub program_as_signer_bump: i16,
    /// Buyer price in lamports
    pub buyer_price: i64,
    /// Token size (usually 1)
    pub token_size: i64,
    /// Timestamp when 'Sell' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `execute_sale_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct ExecuteSaleInstruction<'a> {
    /// Buyer walllet address
    pub buyer: Cow<'a, str>,
    /// seller wallet address
    pub seller: Cow<'a, str>,
    /// Nft Token account address
    pub token_account: Cow<'a, str>,
    /// Token mint address
    pub token_mint: Cow<'a, str>,
    /// Metadata account address
    pub metadata: Cow<'a, str>,
    /// Treasury mint address
    pub treasury_mint: Cow<'a, str>,
    /// Escrow payment account address
    pub escrow_payment_account: Cow<'a, str>,
    /// Seller payment receipt account address
    pub seller_payment_receipt_account: Cow<'a, str>,
    /// Buyer receipt token account addres
    pub buyer_receipt_token_account: Cow<'a, str>,
    /// Authority account address
    pub authority: Cow<'a, str>,
    /// Auction house program address
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account address
    pub auction_house_fee_account: Cow<'a, str>,
    /// Auction house treasury account address
    pub auction_house_treasury: Cow<'a, str>,
    /// Buyer trade state account address
    pub buyer_trade_state: Cow<'a, str>,
    /// Seller trade state account address
    pub seller_trade_state: Cow<'a, str>,
    /// Free trade state account address
    pub free_trade_state: Cow<'a, str>,
    /// Program address signing the transaction
    pub program_as_signer: Cow<'a, str>,
    /// Escrow payment bump
    pub escrow_payment_bump: i16,
    /// Free Trade state bump
    pub free_trade_state_bump: i16,
    /// Program address bump
    pub program_as_signer_bump: i16,
    /// Buyer price in lamports
    pub buyer_price: i64,
    /// Token size (usually 1)
    pub token_size: i64,
    /// Timestamp when 'ExecuteSale' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}
/// A row in the `cancel_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct CancelInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Nft Token account pubkey
    pub token_account: Cow<'a, str>,
    /// Token mint address
    pub token_mint: Cow<'a, str>,
    /// Authority account address
    pub authority: Cow<'a, str>,
    /// Auction house program address
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account address
    pub auction_house_fee_account: Cow<'a, str>,
    /// Trade state account address
    pub trade_state: Cow<'a, str>,
    /// Buyer price in lamports
    pub buyer_price: i64,
    /// Token size (usually 1)
    pub token_size: i64,
    /// Timestamp when 'Cancel' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `deposit_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct DepositInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Wallet used to deposit the funds into account
    pub payment_account: Cow<'a, str>,
    /// Transfer authority pubkey
    pub transfer_authority: Cow<'a, str>,
    /// Escrow account pubkey where funds are deposited
    pub escrow_payment_account: Cow<'a, str>,
    /// Treasury mint pubkey
    pub treasury_mint: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Auction house program pubkey
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// escrow payment bump
    pub escrow_payment_bump: i16,
    /// Amount in lamports deposited
    pub amount: i64,
    /// Timestamp when 'Deposit' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `withdraw_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct WithdrawInstruction<'a> {
    /// wallet address
    pub wallet: Cow<'a, str>,
    /// Receipt account address
    pub receipt_account: Cow<'a, str>,
    /// Escrow account pubkey from where the funds are withdrawn
    pub escrow_payment_account: Cow<'a, str>,
    /// Treasury mint pubkey
    pub treasury_mint: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Auction house program pubkey
    pub auction_house: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// escrow payment bump
    pub escrow_payment_bump: i16,
    /// Amount in lamports withdrawn
    pub amount: i64,
    /// Timestamp when 'Withdraw' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `withdraw_from_fee_instructions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct WithdrawFromFeeInstruction<'a> {
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Wallet where the fee is deposited
    pub fee_withdrawal_destination: Cow<'a, str>,
    /// Auction house fee account pubkey
    pub auction_house_fee_account: Cow<'a, str>,
    /// Auction house program pubkey
    pub auction_house: Cow<'a, str>,
    /// Amount in lamports withdrawn
    pub amount: i64,
    /// Timestamp when 'WithdrawFromFee' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `withdraw_from_treasury` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct WithdrawFromTreasuryInstruction<'a> {
    /// Treasury mint account pubkey
    pub treasury_mint: Cow<'a, str>,
    /// Authority account pubkey
    pub authority: Cow<'a, str>,
    /// Treasury withdrawl wallet pubkey
    pub treasury_withdrawal_destination: Cow<'a, str>,
    /// Auction house treasury account pubkey
    pub auction_house_treasury: Cow<'a, str>,
    /// Auction house program pubkey
    pub auction_house: Cow<'a, str>,
    /// Amount in lamports withdrawn
    pub amount: i64,
    /// Timestamp when 'WithdrawFromTreasury' instruction was received
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
}

/// A row in the `offers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(treat_none_as_null = true)]
#[table_name = "offers"]
pub struct Offer<'a> {
    /// Random Uuid primary key from offers table
    /// Optional so that it can be generated randomly when other fields are inserted into table
    /// Deserialzed as Uuid as id field is primary key so not null
    #[diesel(deserialize_as = "Uuid")]
    pub id: Option<Uuid>,
    /// Trade State account pubkey
    pub trade_state: Cow<'a, str>,
    /// Auction house account pubkey
    pub auction_house: Cow<'a, str>,
    /// Buyer address
    pub buyer: Cow<'a, str>,
    /// Metadata address
    pub metadata: Cow<'a, str>,
    /// Token account address
    pub token_account: Option<Cow<'a, str>>,
    /// Purchase receipt address
    pub purchase_id: Option<Uuid>,
    /// Price
    pub price: i64,
    /// Token size
    pub token_size: i64,
    /// Trade State bump
    pub trade_state_bump: i16,
    /// Created_at timestamp
    pub created_at: NaiveDateTime,
    /// Canceled_at timestamp
    pub canceled_at: Option<NaiveDateTime>,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: Option<i64>,
    /// Marketplace program address
    pub marketplace_program: Cow<'a, str>,
    /// Timestamp when the offer expires
    pub expiry: Option<NaiveDateTime>,
}

/// A row in the `purchases` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct Purchase<'a> {
    /// Random Uuid primary key from offers table
    /// Optional so that it can be generated randomly when other fields are inserted into table
    /// Deserialzed as Uuid as id field is primary key so not null
    #[diesel(deserialize_as = "Uuid")]
    pub id: Option<Uuid>,
    /// Buyer account pubkey
    pub buyer: Cow<'a, str>,
    /// Seller account pubkey
    pub seller: Cow<'a, str>,
    /// Auction House account pubkey
    pub auction_house: Cow<'a, str>,
    /// Metadata
    pub metadata: Cow<'a, str>,
    /// Token size
    pub token_size: i64,
    /// Price
    pub price: i64,
    /// Created at
    pub created_at: NaiveDateTime,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: Option<i64>,
    /// Marketplace program address
    pub marketplace_program: Cow<'a, str>,
}

/// A row in the `listings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, QueryableByName)]
#[diesel(treat_none_as_null = true)]
#[table_name = "listings"]
pub struct Listing<'a> {
    /// Random Uuid primary key from offers table
    /// Optional so that it can be generated randomly when other fields are inserted into table
    /// Deserialzed as Uuid as id field is primary key so not null
    #[diesel(deserialize_as = "Uuid")]
    pub id: Option<Uuid>,
    /// Trade state account pubkey
    pub trade_state: Cow<'a, str>,
    /// Auction House pubkey
    pub auction_house: Cow<'a, str>,
    /// Seller account pubkey
    pub seller: Cow<'a, str>,
    /// Metadata Address
    pub metadata: Cow<'a, str>,
    /// PurchaseReceipt account address
    pub purchase_id: Option<Uuid>,
    /// Price
    pub price: i64,
    /// Token Size
    pub token_size: i64,
    /// Trade State Bump
    pub trade_state_bump: i16,
    /// Created_at timestamp
    pub created_at: NaiveDateTime,
    /// Canceled_at timestamp
    pub canceled_at: Option<NaiveDateTime>,
    /// Solana slot number
    pub slot: i64,
    /// Solana write_version
    pub write_version: Option<i64>,
    /// Marketplace program address
    pub marketplace_program: Cow<'a, str>,
    /// Timestamp when the listing expires
    pub expiry: Option<NaiveDateTime>,
}

/// A row in the `cardinal_entries` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[table_name = "cardinal_entries"]
#[diesel(treat_none_as_null = true)]
pub struct CardinalEntry<'a> {
    /// 'Entry' account pubkey
    pub address: Cow<'a, str>,
    /// 'Namespace' account pubkey
    pub namespace: Cow<'a, str>,
    /// entry name
    pub name: Cow<'a, str>,
    /// wallet pubkey
    pub data: Option<Cow<'a, str>>,
    /// 'ReverseEntry' account pubkey
    pub reverse_entry: Option<Cow<'a, str>>,
    /// Mint address
    pub mint: Cow<'a, str>,
    /// indicates whether the entry is claimed
    pub is_claimed: bool,
    /// Solana slot number
    pub slot: i64,
    /// Solana write version
    pub write_version: i64,
}

/// A row in the `cardinal_namespaces` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct CardinalNamespace<'a> {
    /// 'CardinalNamespace' account pubkey
    pub address: Cow<'a, str>,
    /// Namespace name
    pub name: Cow<'a, str>,
    /// update authority pubkey
    pub update_authority: Cow<'a, str>,
    /// rent authority pubkey
    pub rent_authority: Cow<'a, str>,
    /// approve authority pubkey
    pub approve_authority: Option<Cow<'a, str>>,
    /// Schema
    pub schema: i16,
    /// Daily payment amount
    pub payment_amount_daily: i64,
    /// Spl mint address
    pub payment_mint: Cow<'a, str>,
    /// minimum rental seconds
    pub min_rental_seconds: i64,
    /// maximum rental seconds
    pub max_rental_seconds: Option<i64>,
    /// indicates whether namespace entries can be transfered
    pub transferable_entries: bool,
    /// Solana slot number
    pub slot: i64,
    /// Solana write version
    pub write_version: i64,
}

/// A row in the `geno_habitat_datas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs, clippy::struct_excessive_bools)]
pub struct GenoHabitatData<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    pub habitat_mint: Cow<'a, str>,
    pub level: i16,
    pub element: i16,
    pub genesis: bool,
    pub renewal_timestamp: NaiveDateTime,
    pub expiry_timestamp: NaiveDateTime,
    pub next_day_timestamp: NaiveDateTime,
    pub crystals_refined: i16,
    pub harvester_bytes: Cow<'a, [u8]>,
    pub ki_harvested: i64,
    pub seeds_spawned: bool,
    pub is_sub_habitat: bool,
    pub parent_habitat: Option<Cow<'a, str>>,
    pub sub_habitat_0: Option<Cow<'a, str>>,
    pub sub_habitat_1: Option<Cow<'a, str>>,
    pub harvester_royalty_bips: i32,
    pub harvester_open_market: bool,
    pub total_ki_harvested: i64,
    pub total_crystals_refined: i64,
    pub terraforming_habitat: Option<Cow<'a, str>>,
    pub active: bool,
    pub durability: i32,
    pub habitats_terraformed: i32,
    pub sequence: i64,
    pub guild: Option<i32>,
    pub sub_habitat_cooldown_timestamp: NaiveDateTime,
    pub harvester_settings_cooldown_timestamp: NaiveDateTime,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub harvester: Cow<'a, str>,
    pub daily_ki_harvesting_cap: BigDecimal,
    pub ki_available_to_harvest: Option<BigDecimal>,
    pub has_max_ki: Option<bool>,
}

/// A row in the `geno_rental_agreements` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct GenoRentalAgreement<'a> {
    /// The address of the `HabitatData` this rental agreement belongs to
    pub habitat_address: Cow<'a, str>,
    pub alchemist: Option<Cow<'a, str>>,
    pub rental_period: i64,
    pub rent: i64,
    pub rent_token: Cow<'a, str>,
    pub rent_token_decimals: i16,
    pub last_rent_payment: NaiveDateTime,
    pub next_payment_due: NaiveDateTime,
    pub grace_period: i64,
    pub open_market: bool,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct Governance<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub realm: Cow<'a, str>,
    pub governed_account: Cow<'a, str>,
    pub proposals_count: i64,
    pub reserved: Cow<'a, [u8]>,
    pub voting_proposal_count: i16,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct GovernanceConfig<'a> {
    pub governance_address: Cow<'a, str>,
    pub vote_threshold_type: VoteThresholdEnum,
    pub vote_threshold_percentage: i16,
    pub min_community_weight_to_create_proposal: BigDecimal,
    pub min_instruction_hold_up_time: i64,
    pub max_voting_time: i64,
    pub vote_tipping: VoteTippingEnum,
    pub proposal_cool_off_time: i64,
    pub min_council_weight_to_create_proposal: i64,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct Realm<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub community_mint: Cow<'a, str>,
    pub reserved: Cow<'a, [u8]>,
    pub voting_proposal_count: i16,
    pub authority: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct RealmConfig<'a> {
    pub realm_address: Cow<'a, str>,
    pub use_community_voter_weight_addin: bool,
    pub use_max_community_voter_weight_addin: bool,
    pub reserved: Cow<'a, [u8]>,
    pub min_community_weight_to_create_governance: BigDecimal,
    pub community_mint_max_vote_weight_source: MintMaxVoteEnum,
    pub community_mint_max_vote_weight: i64,
    pub council_mint: Option<Cow<'a, str>>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct RealmConfigAccount<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub realm: Cow<'a, str>,
    pub community_voter_weight_addin: Option<Cow<'a, str>>,
    pub max_community_voter_weight_addin: Option<Cow<'a, str>>,
    pub council_voter_weight_addin: Option<Cow<'a, str>>,
    pub council_max_vote_weight_addin: Option<Cow<'a, str>>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "vote_records_v1"]
#[allow(missing_docs)]
pub struct VoteRecordV1<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub proposal: Cow<'a, str>,
    pub governing_token_owner: Cow<'a, str>,
    pub is_relinquished: bool,
    pub vote_type: VoteWeightV1Enum,
    pub vote_weight: i64,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "vote_records_v2"]
#[allow(missing_docs)]
pub struct VoteRecordV2<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub proposal: Cow<'a, str>,
    pub governing_token_owner: Cow<'a, str>,
    pub is_relinquished: bool,
    pub voter_weight: i64,
    pub vote: VoteRecordV2VoteEnum,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, QueryableByName)]
pub struct VoteRecord {
    #[sql_type = "VarChar"]
    pub address: String,
    #[sql_type = "GovernanceAccountType"]
    pub account_type: GovernanceAccountTypeEnum,
    #[sql_type = "VarChar"]
    pub proposal: String,
    #[sql_type = "VarChar"]
    pub governing_token_owner: String,
    #[sql_type = "Bool"]
    pub is_relinquished: bool,
    #[sql_type = "Nullable<VoteWeightV1>"]
    pub vote_type: Option<VoteWeightV1Enum>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub vote_weight: Option<i64>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub voter_weight: Option<i64>,
    #[sql_type = "Nullable<VoteRecordV2Vote>"]
    pub vote: Option<VoteRecordV2VoteEnum>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "vote_record_v2_vote_approve_vote_choices"]
#[allow(missing_docs)]
pub struct VoteChoice<'a> {
    pub vote_record_v2_address: Cow<'a, str>,
    pub rank: i16,
    pub weight_percentage: i16,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "token_owner_records"]
#[allow(missing_docs)]
pub struct TokenOwnerRecord<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub realm: Cow<'a, str>,
    pub governing_token_mint: Cow<'a, str>,
    pub governing_token_owner: Cow<'a, str>,
    pub governing_token_deposit_amount: i64,
    pub unrelinquished_votes_count: i64,
    pub total_votes_count: i64,
    pub outstanding_proposal_count: i16,
    pub reserved: Cow<'a, [u8]>,
    pub governance_delegate: Option<Cow<'a, str>>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "signatory_records"]
#[allow(missing_docs)]
pub struct SignatoryRecord<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub proposal: Cow<'a, str>,
    pub signatory: Cow<'a, str>,
    pub signed_off: bool,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, QueryableByName)]
pub struct SplGovernanceProposal {
    /// Union of `ProposalV1` and `ProposalV2`
    #[sql_type = "VarChar"]
    pub address: String,
    #[sql_type = "GovernanceAccountType"]
    pub account_type: GovernanceAccountTypeEnum,
    #[sql_type = "VarChar"]
    pub governance: String,
    #[sql_type = "VarChar"]
    pub governing_token_mint: String,
    #[sql_type = "ProposalState"]
    pub state: ProposalStateEnum,
    #[sql_type = "VarChar"]
    pub token_owner_record: String,
    #[sql_type = "diesel::sql_types::SmallInt"]
    pub signatories_count: i16,
    #[sql_type = "diesel::sql_types::SmallInt"]
    pub signatories_signed_off_count: i16,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub yes_votes_count: Option<i64>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub no_votes_count: Option<i64>,
    #[sql_type = "Nullable<diesel::sql_types::SmallInt>"]
    pub instructions_executed_count: Option<i16>,
    #[sql_type = "Nullable<diesel::sql_types::SmallInt>"]
    pub instructions_count: Option<i16>,
    #[sql_type = "Nullable<diesel::sql_types::SmallInt>"]
    pub instructions_next_index: Option<i16>,
    #[sql_type = "Timestamptz"]
    pub draft_at: NaiveDateTime,
    #[sql_type = "Nullable<Timestamptz>"]
    pub signing_off_at: Option<NaiveDateTime>,
    #[sql_type = "Nullable<Timestamptz>"]
    pub voting_at: Option<NaiveDateTime>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub voting_at_slot: Option<i64>,
    #[sql_type = "Nullable<Timestamptz>"]
    pub voting_completed_at: Option<NaiveDateTime>,
    #[sql_type = "Nullable<Timestamptz>"]
    pub executing_at: Option<NaiveDateTime>,
    #[sql_type = "Nullable<Timestamptz>"]
    pub closed_at: Option<NaiveDateTime>,
    #[sql_type = "InstructionExecutionFlags"]
    pub execution_flags: InstructionExecutionFlagsEnum,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub max_vote_weight: Option<i64>,
    #[sql_type = "Nullable<VoteThresholdType>"]
    pub vote_threshold_type: Option<VoteThresholdEnum>,
    #[sql_type = "Nullable<diesel::sql_types::SmallInt>"]
    pub vote_threshold_percentage: Option<i16>,
    #[sql_type = "VarChar"]
    pub name: String,
    #[sql_type = "VarChar"]
    pub description_link: String,
    #[sql_type = "Nullable<ProposalVoteType>"]
    pub vote_type: Option<ProposalVoteTypeEnum>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub deny_vote_weight: Option<i64>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub veto_vote_weight: Option<i64>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub abstain_vote_weight: Option<i64>,
    #[sql_type = "Nullable<Timestamptz>"]
    pub start_voting_at: Option<NaiveDateTime>,
    #[sql_type = "Nullable<diesel::sql_types::BigInt>"]
    pub max_voting_time: Option<i64>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "proposals_v1"]
#[allow(missing_docs)]
pub struct ProposalV1<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub governance: Cow<'a, str>,
    pub governing_token_mint: Cow<'a, str>,
    pub state: ProposalStateEnum,
    pub token_owner_record: Cow<'a, str>,
    pub signatories_count: i16,
    pub signatories_signed_off_count: i16,
    pub yes_votes_count: i64,
    pub no_votes_count: i64,
    pub instructions_executed_count: i16,
    pub instructions_count: i16,
    pub instructions_next_index: i16,
    pub draft_at: NaiveDateTime,
    pub signing_off_at: Option<NaiveDateTime>,
    pub voting_at: Option<NaiveDateTime>,
    pub voting_at_slot: Option<i64>,
    pub voting_completed_at: Option<NaiveDateTime>,
    pub executing_at: Option<NaiveDateTime>,
    pub closed_at: Option<NaiveDateTime>,
    pub execution_flags: InstructionExecutionFlagsEnum,
    pub max_vote_weight: Option<i64>,
    pub vote_threshold_type: Option<VoteThresholdEnum>,
    pub vote_threshold_percentage: Option<i16>,
    pub name: Cow<'a, str>,
    pub description_link: Cow<'a, str>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "proposals_v2"]
#[allow(missing_docs)]
pub struct ProposalV2<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub governance: Cow<'a, str>,
    pub governing_token_mint: Cow<'a, str>,
    pub state: ProposalStateEnum,
    pub token_owner_record: Cow<'a, str>,
    pub signatories_count: i16,
    pub signatories_signed_off_count: i16,
    pub vote_type: ProposalVoteTypeEnum,
    pub deny_vote_weight: Option<i64>,
    pub veto_vote_weight: Option<i64>,
    pub abstain_vote_weight: Option<i64>,
    pub start_voting_at: Option<NaiveDateTime>,
    pub draft_at: NaiveDateTime,
    pub signing_off_at: Option<NaiveDateTime>,
    pub voting_at: Option<NaiveDateTime>,
    pub voting_at_slot: Option<i64>,
    pub voting_completed_at: Option<NaiveDateTime>,
    pub executing_at: Option<NaiveDateTime>,
    pub closed_at: Option<NaiveDateTime>,
    pub execution_flags: InstructionExecutionFlagsEnum,
    pub max_vote_weight: Option<i64>,
    pub max_voting_time: Option<i64>,
    pub vote_threshold_type: Option<VoteThresholdEnum>,
    pub vote_threshold_percentage: Option<i16>,
    pub name: Cow<'a, str>,
    pub description_link: Cow<'a, str>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "proposal_vote_type_multi_choices"]
#[allow(missing_docs)]
pub struct MultiChoice<'a> {
    pub address: Cow<'a, str>,
    pub max_voter_options: i16,
    pub max_winning_options: i16,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "proposal_options"]
#[allow(missing_docs)]
pub struct ProposalOption<'a> {
    pub proposal_address: Cow<'a, str>,
    pub label: Cow<'a, str>,
    pub vote_weight: i64,
    pub vote_result: OptionVoteResultEnum,
    pub transactions_executed_count: i16,
    pub transactions_count: i16,
    pub transactions_next_index: i16,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct ProposalTransaction<'a> {
    pub address: Cow<'a, str>,
    pub account_type: GovernanceAccountTypeEnum,
    pub proposal: Cow<'a, str>,
    pub option_index: i16,
    pub transaction_index: i16,
    pub hold_up_time: i64,
    pub executed_at: Option<NaiveDateTime>,
    pub execution_status: TransactionExecutionStatusEnum,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
    pub program_id: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct ProposalTransactionInstruction<'a> {
    pub proposal_transaction: Cow<'a, str>,
    pub program_id: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct ProposalTransactionInstructionAccount<'a> {
    pub proposal_transaction: Cow<'a, str>,
    pub account_pubkey: Cow<'a, str>,
    pub is_signer: bool,
    pub is_writable: bool,
    /// The slot number of this account's last known update
    pub slot: i64,
    /// The write version of this account's last known update
    pub write_version: i64,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct Collection<'a> {
    pub id: Cow<'a, str>,
    pub image: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub twitter_url: Option<Cow<'a, str>>,
    pub discord_url: Option<Cow<'a, str>>,
    pub website_url: Option<Cow<'a, str>>,
    pub magic_eden_id: Option<Cow<'a, str>>,
    pub verified_collection_address: Option<Cow<'a, str>>,
    pub pieces: i64,
    pub verified: bool,
    pub go_live_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct CollectionMint<'a> {
    pub collection_id: Cow<'a, str>,
    pub mint: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub image: Cow<'a, str>,
    pub created_at: NaiveDateTime,
    pub rank: i64,
    pub rarity: BigDecimal,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[allow(missing_docs)]
pub struct CollectionMintAttribute<'a> {
    pub mint: Cow<'a, str>,
    pub attribute: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub value_perc: BigDecimal,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "dolphin_stats"]
#[allow(missing_docs)]
pub struct DolphinStats<'a> {
    pub collection_symbol: Cow<'a, str>,
    pub floor_1d: i64,
    pub floor_7d: i64,
    pub floor_30d: i64,
    pub listed_1d: i64,
    pub listed_7d: i64,
    pub listed_30d: i64,
    pub volume_1d: i64,
    pub volume_7d: i64,
    pub volume_30d: i64,
    pub last_floor_1d: i64,
    pub last_floor_7d: i64,
    pub last_floor_30d: i64,
    pub last_listed_1d: i64,
    pub last_listed_7d: i64,
    pub last_listed_30d: i64,
    pub last_volume_1d: i64,
    pub last_volume_7d: i64,
    pub last_volume_30d: i64,
}

#[derive(Debug, Clone, AsChangeset)]
#[table_name = "dolphin_stats"]
#[allow(missing_docs)]
pub struct DolphinStats1D<'a> {
    pub collection_symbol: Cow<'a, str>,
    pub floor_1d: i64,
    pub listed_1d: i64,
    pub volume_1d: i64,
    pub last_floor_1d: i64,
    pub last_listed_1d: i64,
    pub last_volume_1d: i64,
}
