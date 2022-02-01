//! Models to query and insert data according to the Diesel schema DSLs

// Queryable and Insertable are imported globally from diesel

// TODO: do we need chrono instead of std::time?
use std::borrow::Cow;

use chrono::NaiveDateTime;

use super::schema::{
    attributes, bids, editions, files, listing_metadatas, listings, master_editions,
    metadata_collections, metadata_creators, metadata_jsons, metadatas, storefronts,
    token_accounts,
};

/// A row in the `bids` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[belongs_to(parent = "Listing<'_>", foreign_key = "listing_address")]
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
#[belongs_to(parent = "Metadata<'_>", foreign_key = "metadata_address")]
pub struct Edition<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The address of this edition's parent master edition
    pub parent_address: Cow<'a, str>,
    /// The ordinal of this edition
    pub edition: i64,
    /// The metadata this edition refers to
    pub metadata_address: Cow<'a, str>,
}

/// A row in the `listing_metadatas` table.  This is a join on `listings` and
/// `metadatas`
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[belongs_to(parent = "Listing<'_>", foreign_key = "listing_address")]
#[belongs_to(parent = "Metadata<'_>", foreign_key = "metadata_address")]
pub struct ListingMetadata<'a> {
    /// The address of this record's listing
    pub listing_address: Cow<'a, str>,
    /// The address of this record's metadata
    pub metadata_address: Cow<'a, str>,
    /// The index of the metadata in the array of items for the listing
    pub metadata_index: i32,
}

/// A row in the `listings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[belongs_to(parent = "Storefront<'_>", foreign_key = "store_owner")]
pub struct Listing<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The timestamp this auction ends at, if applicable
    pub ends_at: Option<NaiveDateTime>,
    /// The timestamp this auction was created at
    pub created_at: NaiveDateTime,
    /// Whether this auction has ended
    pub ended: bool,
    /// The authority of this auction
    pub authority: Cow<'a, str>,
    /// The item being auctioned
    pub token_mint: Cow<'a, str>,
    /// The owner of the store this auction was found from
    pub store_owner: Cow<'a, str>,
    /// The amount of the highest bid, if applicable
    pub highest_bid: Option<i64>,
    /// The gap time of the auction, if applicable
    pub end_auction_gap: Option<NaiveDateTime>,
    /// The starting bid of the auction, if applicable
    pub price_floor: Option<i64>,
    /// The total number of live bids on this auction, if applicable
    pub total_uncancelled_bids: Option<i32>,
    /// The minimum bid increase in percentage points during the ending gap of
    /// the auction, if applicable
    pub gap_tick_size: Option<i32>,
    /// The price of the listing, if an instant sale
    pub instant_sale_price: Option<i64>,
    /// The name of the listing
    pub name: Cow<'a, str>,
    /// The timestamp of the last bid, if applicable and the auction has bids
    pub last_bid_time: Option<NaiveDateTime>,
}

/// A row in the `master_editions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Associations)]
#[belongs_to(parent = "Metadata<'_>", foreign_key = "metadata_address")]
pub struct MasterEdition<'a> {
    /// The address of this account
    pub address: Cow<'a, str>,
    /// The available printing supply of the master edition
    pub supply: i64,
    /// The maximum printing supply of the master edition, or `None` if it is
    /// unlimited
    pub max_supply: Option<i64>,
    /// The metadata this edition refers to
    pub metadata_address: Cow<'a, str>,
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
}

/// Join of `metadatas` and `metadata_jsons` for an NFT
#[derive(Debug, Clone, Queryable)]
pub struct Nft {
    // Table metadata

    /// The address of this account
    pub address: String,
    /// The name of this item
    pub name: String,

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
