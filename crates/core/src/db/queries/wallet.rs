//! Query utilities for looking up  wallets.
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Integer, Nullable, Text},
};

use crate::{
    db::{
        models::{CollectedCollection, CreatedCollection, WalletActivity, ReadOffer},
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
buyer, th.twitter_handle as buyer_twitter_handles, trade_state, token_account, purchase_id,
token_size, trade_state_bump, canceled_at, slot, write_version, expiry
FROM offers
    LEFT JOIN twitter_handle_name_services th on (th.wallet_address = offers.buyer)
    WHERE buyer = $1
    AND offers.purchase_id IS NULL
    AND offers.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('OFFER_PLACED' = ANY($2) OR $2 IS NULL)
UNION
SELECT offers.id as id,  metadata, price, auction_house, created_at, marketplace_program,
buyer, th.twitter_handle as buyer_twitter_handles, trade_state, token_account, purchase_id,
token_size, trade_state_bump, canceled_at, slot, write_version, expiry
FROM offers
    LEFT JOIN twitter_handle_name_services th on (th.wallet_address = offers.buyer)
    LEFT JOIN metadatas on (metadatas.address = offers.metadata)
    LEFT JOIN current_metadata_owners on (current_metadata_owners.mint_address = metadatas.mint_address)
    WHERE current_metadata_owners.owner_address = $1
    AND offers.purchase_id IS NULL
    AND offers.auction_house != '3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y'
    AND ('OFFER_RECEIVED' = ANY($2) OR $2 IS NULL)
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
    offers_type: impl ToSql<Nullable<Text>, Pg>,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<ReadOffer>> {
    diesel::sql_query(OFFERS_QUERY)
        .bind(address)
        .bind(offers_type)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load wallet offers")
}

const COLLECTED_COLLECTIONS_QUERY: &str = r"
SELECT collection_metadatas.address as collection_nft_address,
	COUNT(metadatas.address) as nfts_owned,
	COALESCE(collection_stats.floor_price * COUNT(metadatas.address), 0) as estimated_value
    FROM metadatas
    INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address)
    INNER JOIN metadata_jsons ON (metadata_jsons.metadata_address = metadatas.address)
    INNER JOIN metadata_collection_keys ON (metadata_collection_keys.metadata_address = metadatas.address)
	INNER JOIN collection_stats ON (metadata_collection_keys.collection_address = collection_stats.collection_address)
    INNER JOIN metadatas collection_metadatas ON (collection_metadatas.mint_address = metadata_collection_keys.collection_address)
	INNER JOIN metadata_jsons collection_metadata_jsons ON (collection_metadata_jsons.metadata_address = collection_metadatas.address)
    WHERE current_metadata_owners.owner_address = $1
    AND metadata_collection_keys.verified
    GROUP BY collection_nft_address, collection_stats.floor_price
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

const CREATED_COLLECTIONS_QUERY: &str = r"
SELECT metadatas.address
    FROM metadatas
    INNER JOIN collection_stats ON (metadatas.mint_address = collection_stats.collection_address)
    WHERE
        metadatas.update_authority_address = $1;
-- $1: address::text";

/// Load created collections for a wallet.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn created_collections(
    conn: &Connection,
    address: impl ToSql<Text, Pg>,
) -> Result<Vec<CreatedCollection>> {
    diesel::sql_query(CREATED_COLLECTIONS_QUERY)
        .bind(address)
        .load(conn)
        .context("Failed to load wallet(s) created collections")
}
