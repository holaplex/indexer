//! Models to query and insert data according to the Diesel schema DSLs

// Queryable and Insertable are imported globally from diesel

use std::borrow::Cow;

use chrono::NaiveDateTime;
use diesel::sql_types::{Array, Bool, Int4, Int8, Nullable, Text, Timestamp, VarChar};

use super::schema::{
    attributes, auction_caches, auction_datas, auction_datas_ext, auction_houses, bid_receipts,
    bids, candy_machine_collection_pdas, candy_machine_config_lines, candy_machine_creators,
    candy_machine_datas, candy_machine_end_settings, candy_machine_gate_keeper_configs,
    candy_machine_hidden_settings, candy_machine_whitelist_mint_settings, candy_machines,
    cardinal_paid_claim_approvers, cardinal_time_invalidators, cardinal_token_manager_invalidators,
    cardinal_token_managers, cardinal_use_invalidators, editions, files, graph_connections,
    listing_metadatas, listing_receipts, master_editions, metadata_collection_keys,
    metadata_collections, metadata_creators, metadata_jsons, metadatas, purchase_receipts,
    store_config_jsons, store_configs, store_creators, storefronts, stores, token_accounts,
    twitter_handle_name_services, whitelisted_creators,
};
use crate::db::custom_types::{EndSettingType, TokenStandardEnum, WhitelistMintMode};

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

/// A row in the `token_accounts` table
/// helpful for tracking exchanges of tokens
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
pub struct TokenAccount<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The mint address of the token
    pub mint_address: Cow<'a, str>,
    /// The owner token
    pub owner_address: Cow<'a, str>,
    /// The amount of the token, often 1
    pub amount: i64,
    /// Solana slot number
    /// The period of time for which each leader ingests transactions and produces a block.
    pub slot: Option<i64>,
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

/// Join of `metadatas` and `metadata_jsons` for an NFT
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

    // Table metadata_json
    /// Metadata description
    #[sql_type = "Nullable<Text>"]
    pub description: Option<String>,

    /// Metadata Image url
    #[sql_type = "Nullable<Text>"]
    pub image: Option<String>,
}

/// Union of `listing_receipts` and `purchase_receipts` for an `NFTActivity`
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct NftActivity {
    /// The address of the activity
    #[sql_type = "VarChar"]
    pub address: String,

    /// The metadata associated of the activity
    #[sql_type = "VarChar"]
    pub metadata: String,

    /// The auction house activity generated from
    #[sql_type = "VarChar"]
    pub auction_house: String,

    /// The price of listing or purchase
    #[sql_type = "Int8"]
    pub price: i64,

    /// Listing/Purchase created time
    #[sql_type = "Timestamp"]
    pub created_at: NaiveDateTime,

    /// The wallet address asociated to the activity [seller, buyer]
    #[sql_type = "Array<VarChar>"]
    pub wallets: Vec<String>,

    /// Listing/Purchase created time
    #[sql_type = "Text"]
    pub activity_type: String,
}

/// Join of `metadatas` `metadata_jsons` `store_creators` for an collection preview
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

    // Table metadata_json
    /// Metadata description
    #[sql_type = "Nullable<Text>"]
    pub description: Option<String>,

    /// Metadata Image url
    #[sql_type = "Nullable<Text>"]
    pub image: Option<String>,
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
    pub fingerprint: Cow<'a, Vec<u8>>,
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
    pub id: Cow<'a, uuid::Uuid>,
    /// Address of metadata first verified creator
    pub first_verified_creator: Option<Cow<'a, str>>,
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
    /// Auction house account address
    pub auction_house_address: Cow<'a, str>,
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
}

/// A row in the `listing_receipts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
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
    pub address: Cow<'a, str>,
    /// Name
    pub name: Cow<'a, str>,
    /// URI pointing to JSON representing the asset
    pub uri: Cow<'a, str>,
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

/// Join of `metadatas` `metadata_jsons` `store_creators` for an collection preview
#[derive(Debug, Clone, Queryable, QueryableByName)]
pub struct CardinalTokenManagerQuery {
    #[sql_type = "Text"]
    pub address: String,

    // #[sql_type = "Text"]
    // pub token_manager_address: String,
    #[sql_type = "Nullable<Text>"]
    pub payment_mint: Option<String>,
    /* #[sql_type = "Timestamp"]
     * pub state_changed_at: NaiveDateTime, */
}

/// A row in the `token_managers` table
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

/// A row in the `token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_token_manager_invalidators"]
pub struct CardinalTokenManagerInvalidator<'a> {
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Address of an active invalidator for this token_manager
    pub invalidator: Cow<'a, str>,
}

/// A row in the `token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_time_invalidators"]
pub struct CardinalTimeInvalidator<'a> {
    /// Address of the time_invalidator
    pub address: Cow<'a, str>,
    /// Bump seed of the time_invalidator
    pub bump: i16,
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Optional expiration which this time invalidator will expire
    pub expiration: Option<NaiveDateTime>,
    /// Duration after claim
    pub duration_seconds: Option<i64>,
    /// Amount the pay for extension
    pub extension_payment_amount: Option<i64>,
    /// Duration received after extension
    pub extension_duration_seconds: Option<i64>,
    /// Mint that extension is denominated in
    pub extension_payment_mint: Option<Cow<'a, str>>,
    /// Optional max this can ever be extended until
    pub max_expiration: Option<NaiveDateTime>,
    /// Whether extension can be in partial increments
    pub disable_partial_extension: Option<bool>,
}

/// A row in the `token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_use_invalidators"]
pub struct CardinalUseInvalidator<'a> {
    /// Address of the use_invalidator
    pub address: Cow<'a, str>,
    /// Bump seed of the use_invalidator
    pub bump: i16,
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Optional expiration which this time invalidator will expire
    pub usages: i64,
    /// Address that can increment usages
    pub use_authority: Option<Cow<'a, str>>,
    /// Total usages
    pub total_usages: Option<i64>,
    /// Amount the pay for extension
    pub extension_payment_amount: Option<i64>,
    /// Mint that extension is denominated in
    pub extension_payment_mint: Option<Cow<'a, str>>,
    /// Number of usages received after extension
    pub extension_usages: Option<i64>,
    /// Optional max this can ever be extended until
    pub max_usages: Option<i64>,
}

/// A row in the `token_manager_invalidators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[table_name = "cardinal_paid_claim_approvers"]
pub struct CardinalPaidClaimApprover<'a> {
    /// Address of the use_invalidator
    pub address: Cow<'a, str>,
    /// Bump seed of the use_invalidator
    pub bump: i16,
    /// Address of the token_manager
    pub token_manager_address: Cow<'a, str>,
    /// Amount the pay for extension
    pub payment_amount: i64,
    /// Mint that extension is denominated in
    pub payment_mint: Cow<'a, str>,
    /// Address that can collect rent
    pub collector: Cow<'a, str>,
}
