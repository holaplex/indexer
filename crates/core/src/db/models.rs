//! Models to query and insert data according to the Diesel schema DSLs

// Queryable and Insertable are imported globally from diesel

// TODO: do we need chrono instead of std::time?
use std::borrow::Cow;

use chrono::NaiveDateTime;

use super::schema::{
    editions, listing_metadatas, listings, master_editions, metadata_creators, metadatas,
};

/// A row in the `editions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct Edition<'a> {
    /// The address of this account
    pub address: Cow<'a, [u8]>,
    /// The address of this edition's parent master edition
    pub parent_address: Cow<'a, [u8]>,
    /// The ordinal of this edition
    pub edition: i64,
}

/// A row in the `listing_metadatas` table.  This is a join on `listings` and
/// `metadatas`
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct ListingMetadata<'a> {
    /// The address of this record's listing
    pub listing_address: Cow<'a, [u8]>,
    /// The address of this record's metadata
    pub metadata_address: Cow<'a, [u8]>,
}

/// A row in the `listings` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct Listing<'a> {
    /// The address of this account
    pub address: Cow<'a, [u8]>,
    /// The timestamp this auction ends at, if applicable
    pub ends_at: Option<NaiveDateTime>,
    /// The timestamp this auction was created at
    pub created_at: NaiveDateTime,
    /// Whether this auction has ended
    pub ended: bool,
    /// The authority of this auction
    pub authority: Cow<'a, [u8]>,
    /// The item being auctioned
    pub token_mint: Cow<'a, [u8]>,
    /// The store this auction was found from
    pub store: Cow<'a, [u8]>,
    /// The amount of the last bid, if applicable
    pub last_bid: Option<i64>,
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
}

/// A row in the `master_editions` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct MasterEdition<'a> {
    /// The address of this account
    pub address: Cow<'a, [u8]>,
    /// The available printing supply of the master edition
    pub supply: i64,
    /// The maximum printing supply of the master edition, or `None` if it is
    /// unlimited
    pub max_supply: Option<i64>,
}

/// A row in the `metadata_creators` table.  This is a join on `metadatas` and
/// creator wallets.
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct MetadataCreator<'a> {
    /// The address of this record's metadata
    pub metadata_address: Cow<'a, [u8]>,
    /// The address of this record's creator wallet
    pub creator_address: Cow<'a, [u8]>,
    /// The share of the creator, in percentage points
    pub share: i32,
    /// Whether this creator has verified this metadata
    pub verified: bool,
}

/// A row in the `metadatas` table
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
pub struct Metadata<'a> {
    /// The address of this account
    pub address: Cow<'a, [u8]>,
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
    pub update_authority_address: Cow<'a, [u8]>,
    /// The token address for this item
    pub mint_address: Cow<'a, [u8]>,
    /// True if this item is in the secondary market.  Immutable once set.
    pub primary_sale_happened: bool,
    /// True if this item can be changed by the update authority
    pub is_mutable: bool,
    /// Metaplex isn't clear about what this is.  Assume reserved.
    pub edition_nonce: Option<i32>,
}
