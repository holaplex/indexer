//! Models to query and insert data according to the Diesel schema DSLs

// Queryable and Insertable are imported globally from diesel

use std::borrow::Cow;

use chrono::NaiveDateTime;

use super::schema::{
    attributes, auction_caches, auction_datas, auction_datas_ext, auction_houses, bids, editions,
    files, listing_metadatas, master_editions, metadata_collections, metadata_creators,
    metadata_jsons, metadatas, store_config_jsons, store_configs, store_denylist, storefronts,
    stores, token_accounts, token_transfers, whitelisted_creators,
};

/// A row in the `bids` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
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
}

/// A row in the `token_accounts` table
/// helpful for tracking exchanges of tokens
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct TokenAccount<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The mint address of the token
    pub mint_address: Cow<'a, str>,
    /// The owner token
    pub owner_address: Cow<'a, str>,
    /// The amount of the token, often 1
    pub amount: i64,
    /// updated_at
    pub updated_at: NaiveDateTime,
}

/// A row in the `metadatas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
}

/// A row in the `storefronts` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
#[derive(Debug, Clone, Queryable)]
pub struct Nft {
    // Table metadata
    /// The address of this account
    pub address: String,

    /// The name of this item
    pub name: String,

    /// The royalty percentage of the creator, in basis points (0.01%, values
    /// range from 0-10,000)
    pub seller_fee_basis_points: i32,

    /// The token address for this item
    pub mint_address: String,

    /// True if this item is in the secondary market.  Immutable once set.
    pub primary_sale_happened: bool,

    // Table metadata_json
    /// Metadata description
    pub description: Option<String>,

    /// Metadata Image url
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
pub struct MetadataJson<'a> {
    /// Metadata Address
    pub metadata_address: Cow<'a, str>,
    /// Metadata URI fingerprint - Cid for Ipfs and ArTxid for Arweave
    pub fingerprint: Cow<'a, Vec<u8>>,
    /// Metadata Timestamp
    pub updated_at: NaiveDateTime,
    /// Metadata description
    pub description: Option<Cow<'a, str>>,
    /// Metadata Image url
    pub image: Option<Cow<'a, str>>,
    /// Metadata Animation url
    pub animation_url: Option<Cow<'a, str>>,
    /// Metadata External Url
    pub external_url: Option<Cow<'a, str>>,
    /// Metadata Category
    pub category: Option<Cow<'a, str>>,
    /// Metadata URI raw json
    pub raw_content: Cow<'a, serde_json::Value>,
}

/// A row in the `files` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
#[table_name = "attributes"]
pub struct MetadataAttributeWrite<'a> {
    /// Metadata address
    pub metadata_address: Cow<'a, str>,
    /// Attribute value
    pub value: Option<Cow<'a, str>>,
    /// Attribute trait type
    pub trait_type: Option<Cow<'a, str>>,
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
}

/// A row in the `metadata_collections` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
pub struct StoreConfig<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// Store settings URI
    pub settings_uri: Option<Cow<'a, str>>,
}

/// A row in the `whitelisted_creators` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
}

/// A row in the `auction_houses` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
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
}

/// A row in the `token_transfers` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct TokenTransfer<'a> {
    /// Address of the wallet from which NFT was transferred
    pub owner_from: Cow<'a, str>,
    /// Address of the wallet to which NFT was transferred
    pub owner_to: Cow<'a, str>,
    /// Mint address of the token
    pub mint_address: Cow<'a, str>,
    /// Time at which transfer occurred
    ///
    /// This is an approximate time so the block time of the transaction
    /// signature can be different.
    pub transferred_at: NaiveDateTime,
}
