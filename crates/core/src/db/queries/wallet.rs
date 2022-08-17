//! Query utilities for looking up  wallets.
use diesel::{pg::Pg, prelude::*, serialize::ToSql, sql_types::Text};

use crate::{
    db::{
        models::{Nft, WalletActivity},
        Connection,
    },
    error::prelude::*,
};

const ACTIVITES_QUERY: &str = r"
SELECT listings.id as id, metadata, auction_house, price, auction_house, created_at,
array[seller] as wallets,
array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
'listing' as activity_type
    FROM listings
    LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listings.seller)
    WHERE seller = $1
    AND canceled_at IS NULL
UNION
SELECT listings.id as id, metadata, auction_house, price, auction_house, canceled_at as created_at,
array[seller] as wallets,
array[twitter_handle_name_services.twitter_handle] as wallet_twitter_handles,
'cancel_listing' as activity_type
    FROM listings
    LEFT JOIN twitter_handle_name_services on (twitter_handle_name_services.wallet_address = listings.seller)
    WHERE seller = $1
    AND canceled_at IS NOT NULL
UNION
SELECT purchases.id as id, metadata, auction_house, price, auction_house, created_at,
array[seller, buyer] as wallets,
array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
'purchase' as activity_type
    FROM purchases
    LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchases.seller)
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchases.buyer)
    WHERE buyer = $1
UNION
SELECT purchases.id as id, metadata, auction_house, price, auction_house, created_at,
array[seller, buyer] as wallets,
array[sth.twitter_handle, bth.twitter_handle] as wallet_twitter_handles,
'sell' as activity_type
    FROM purchases
    LEFT JOIN twitter_handle_name_services sth on (sth.wallet_address = purchases.seller)
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = purchases.buyer)
    WHERE seller = $1
UNION
SELECT offers.id as id, metadata, auction_house, price, auction_house, created_at,
array[buyer] as wallets,
array[bth.twitter_handle] as wallet_twitter_handles,
'offer' as activity_type
    FROM offers
    LEFT JOIN twitter_handle_name_services bth on (bth.wallet_address = offers.buyer)
    WHERE buyer = $1
    AND offers.purchase_id IS NULL
ORDER BY created_at DESC;
-- $1: address::text";

/// Load listing and sales activity for wallets.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn activities(conn: &Connection, address: impl ToSql<Text, Pg>) -> Result<Vec<WalletActivity>> {
    diesel::sql_query(ACTIVITES_QUERY)
        .bind(address)
        .load(conn)
        .context("Failed to load wallet(s) activities")
}

const COLLECTED_COLLECTIONS_QUERY: &str = r"
SELECT
    metadatas.address,
    metadatas.name,
    metadatas.seller_fee_basis_points,
    metadatas.update_authority_address,
    metadatas.mint_address,
    metadatas.primary_sale_happened,
    metadatas.uri,
    metadatas.slot,
    metadata_jsons.description,
    metadata_jsons.image,
    metadata_jsons.animation_url,
    metadata_jsons.external_url,
    metadata_jsons.category,
    metadata_jsons.model,
    current_metadata_owners.token_account_address
        FROM metadatas
        INNER JOIN metadata_jsons ON (metadata_jsons.metadata_address = metadatas.address)
        INNER JOIN metadata_collection_keys ON (metadata_collection_keys.collection_address = metadatas.mint_address)
        INNER JOIN current_metadata_owners ON (current_metadata_owners.mint_address = metadatas.mint_address)
        WHERE current_metadata_owners.owner_address = $1
    -- $1: address::text";

/// Load collected collections for a wallet.
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn collected_collections(conn: &Connection, address: impl ToSql<Text, Pg>) -> Result<Vec<Nft>> {
    diesel::sql_query(COLLECTED_COLLECTIONS_QUERY)
        .bind(address)
        .load(conn)
        .context("Failed to load wallet(s) collected collections")
}
