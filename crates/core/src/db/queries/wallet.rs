//! Query utilities for looking up  wallets.
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Integer, Nullable, Text},
};

use crate::{
    db::{
        models::{CollectedCollection, Offer, WalletActivity},
        Connection,
    },
    error::prelude::*,
};

const ACTIVITES_QUERY: &str = r"
SELECT listings.id as id, metadata, price, auction_house, created_at, marketplace_program,
array[seller] as wallets,
array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
'listing' as activity_type
    FROM listings
    LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listings.seller)
    WHERE seller = $1
    AND canceled_at IS NULL
    AND listings.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('LISTINGS' = ANY($2) OR $2 IS NULL)
UNION
SELECT purchases.id as id, metadata, price, auction_house, created_at, marketplace_program,
array[seller, buyer] as wallets,
array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
'purchase' as activity_type
    FROM purchases
    LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchases.seller)
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchases.buyer)
    WHERE buyer = $1
    AND ('PURCHASES' = ANY($2) OR $2 IS NULL)
UNION
SELECT purchases.id as id, metadata, price, auction_house, created_at, marketplace_program,
array[seller, buyer] as wallets,
array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
'sell' as activity_type
    FROM purchases
    LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchases.seller)
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchases.buyer)
    WHERE seller = $1
    AND ('SALES' = ANY($2) OR $2 IS NULL)
UNION
SELECT offers.id as id, metadata, price, auction_house, created_at, marketplace_program,
array[buyer] as wallets,
array[bth.twitter_handle] as wallet_twitter_handles,
'offer' as activity_type
    FROM offers
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = offers.buyer)
    WHERE buyer = $1
    AND offers.purchase_id IS NULL
    AND offers.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('OFFERS' = ANY($2) OR $2 IS NULL)
ORDER BY created_at DESC
LIMIT $3
OFFSET $4;

-- $1: address::text
-- $2: event_types::text[]
-- $3: limit::integer
-- $4: offset::integer";

/// Load listing, purchase, sales and offer activity for wallets.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn activities(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    event_types: impl ToSql<Nullable<Array<Text>>, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<WalletActivity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(address)
        .bind(event_types)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load wallet(s) activities")
}

const OFFERS_QUERY: &str = r"
SELECT offers.id as id,  metadata, price, auction_house, created_at, marketplace_program,
buyer, trade_state, token_account, purchase_id,
token_size, trade_state_bump, canceled_at, write_version, expiry, offers.slot as slot
FROM offers
    WHERE buyer = $1
    AND offers.purchase_id IS NULL
    AND offers.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('OFFER_PLACED' = $2 OR $2 IS NULL)
UNION
SELECT offers.id as id,  metadata, price, auction_house, created_at, marketplace_program,
buyer, trade_state, token_account, purchase_id,
token_size, trade_state_bump, canceled_at, write_version, expiry, offers.slot as slot
FROM offers
    INNER JOIN metadatas on (metadatas.address = offers.metadata)
    INNER JOIN current_metadata_owners on (current_metadata_owners.mint_address = metadatas.mint_address)
    WHERE current_metadata_owners.owner_address = $1
    AND offers.purchase_id IS NULL
    AND offers.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('OFFER_RECEIVED' = $2 OR $2 IS NULL)
ORDER BY created_at DESC
LIMIT $3
OFFSET $4;

-- $1: address::text
-- $2: offers_type::text
-- $3: limit::integer
-- $4: offset::integer";

/// Load offers for a wallet.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn offers(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
    offer_type: impl ToSql<Nullable<Text>, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Offer>> {
    let result = diesel::sql_query(OFFERS_QUERY)
        .bind(address)
        .bind(offer_type)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load wallet offers");
    println!("Query Result: {result:?}");
    result
}

const COLLECTED_COLLECTIONS_QUERY: &str = r"
SELECT collections.id as collection_id,
	COUNT(metadatas.address) as nfts_owned,
	COALESCE(dolphin_stats.floor_1d * COUNT(metadatas.address), 0) as estimated_value
    FROM collections
    INNER JOIN collection_mints ON (collection_mints.collection_id = collections.id)
	INNER JOIN metadatas ON (metadatas.mint_address = collection_mints.mint)
    INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address)
    INNER JOIN metadata_jsons ON (metadata_jsons.metadata_address = metadatas.address)
    INNER JOIN dolphin_stats ON (dolphin_stats.collection_symbol = collections.id)
    WHERE current_metadata_owners.owner_address = $1
    GROUP BY collections.id, dolphin_stats.floor_1d
	ORDER BY estimated_value DESC;
    -- $1: address::text";

/// Load collected collections for a wallet.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn collected_collections(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
) -> Result<Vec<CollectedCollection>> {
    diesel::sql_query(COLLECTED_COLLECTIONS_QUERY)
        .bind(address)
        .load(conn)
        .context("Failed to load wallet(s) collected collections")
}
