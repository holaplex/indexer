use indexer_core::{
    db::{
        insert_into,
        models::{Listing as DbListing, PublicBid as DbPublicBid, Purchase as DbPurchase},
        tables::{listings, public_bids, purchases},
    },
    prelude::*,
};
use mpl_auction_house::{Listing, PublicBid, Purchase};

use super::Client;
use crate::prelude::*;

pub async fn process_listing(client: &Client, key: Pubkey, listing: Listing) -> Result<()> {
    let row = DbListing {
        address: Owned(bs58::encode(key).into_string()),
        trade_state: Owned(bs58::encode(listing.trade_state).into_string()),
        bookkeeper: Owned(bs58::encode(listing.bookkeeper).into_string()),
        auction_house: Owned(bs58::encode(listing.auction_house).into_string()),
        seller: Owned(bs58::encode(listing.seller).into_string()),
        token_mint: Owned(bs58::encode(listing.token_mint).into_string()),
        price: listing.price.try_into()?,
        token_size: listing.token_size.try_into()?,
        bump: listing.bump.into(),
        trade_state_bump: listing.trade_state_bump.into(),
        activated_at: listing
            .activated_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
        closed_at: listing
            .closed_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
    };

    client
        .db()
        .run(move |db| {
            insert_into(listings::table)
                .values(&row)
                .on_conflict(listings::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert listing!")?;

    Ok(())
}

pub async fn process_purchase(client: &Client, key: Pubkey, purchase: Purchase) -> Result<()> {
    let row = DbPurchase {
        address: Owned(bs58::encode(key).into_string()),
        buyer: Owned(bs58::encode(purchase.buyer).into_string()),
        seller: Owned(bs58::encode(purchase.seller).into_string()),
        auction_house: Owned(bs58::encode(purchase.auction_house).into_string()),
        token_mint: Owned(bs58::encode(purchase.token_mint).into_string()),
        token_size: purchase.token_size.try_into()?,
        price: purchase.price.try_into()?,
        bump: purchase.bump.into(),
        created_at: purchase
            .created_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
    };

    client
        .db()
        .run(move |db| {
            insert_into(purchases::table)
                .values(&row)
                .on_conflict(purchases::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert purchase!")?;

    Ok(())
}

pub async fn process_public_bid(client: &Client, key: Pubkey, public_bid: PublicBid) -> Result<()> {
    let row = DbPublicBid {
        address: Owned(bs58::encode(key).into_string()),
        trade_state: Owned(bs58::encode(public_bid.trade_state).into_string()),
        bookkeeper: Owned(bs58::encode(public_bid.bookkeeper).into_string()),
        auction_house: Owned(bs58::encode(public_bid.auction_house).into_string()),
        wallet: Owned(bs58::encode(public_bid.wallet).into_string()),
        token_mint: Owned(bs58::encode(public_bid.token_mint).into_string()),
        price: public_bid.price.try_into()?,
        token_size: public_bid.token_size.try_into()?,
        bump: public_bid.bump.into(),
        trade_state_bump: public_bid.trade_state_bump.into(),
        activated_at: public_bid
            .activated_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
        closed_at: public_bid
            .closed_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
    };

    client
        .db()
        .run(move |db| {
            insert_into(public_bids::table)
                .values(&row)
                .on_conflict(public_bids::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert public bid!")?;

    Ok(())
}
