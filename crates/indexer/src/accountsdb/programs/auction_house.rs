use anchor_lang::AccountDeserialize;
use mpl_auction_house::{
    AuctionHouse, Listing, PublicBid, Purchase, AUCTION_HOUSE_SIZE, LISTING_SIZE, PUBLIC_BID_SIZE,
    PURCHASE_SIZE,
};

use super::{
    accounts::{auction_house, receipt},
    AccountUpdate, Client,
};
use crate::prelude::*;

async fn process_auction_house(client: &Client, update: AccountUpdate) -> Result<()> {
    let house: AuctionHouse = AuctionHouse::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize auction house data")?;

    auction_house::process(client, update.key, house).await
}

async fn process_listing(client: &Client, update: AccountUpdate) -> Result<()> {
    let listing: Listing = Listing::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize listing data")?;

    receipt::process_listing(client, update.key, listing).await
}
async fn process_public_bid(client: &Client, update: AccountUpdate) -> Result<()> {
    let public_bid: PublicBid = PublicBid::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize public bid data")?;

    receipt::process_public_bid(client, update.key, public_bid).await
}
async fn process_purchase(client: &Client, update: AccountUpdate) -> Result<()> {
    let purchase: Purchase = Purchase::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize purchase data")?;

    receipt::process_purchase(client, update.key, purchase).await
}
pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        AUCTION_HOUSE_SIZE => process_auction_house(client, update).await,
        LISTING_SIZE => process_listing(client, update).await,
        PUBLIC_BID_SIZE => process_public_bid(client, update).await,
        PURCHASE_SIZE => process_purchase(client, update).await,
        _ => {
            return Ok(());
        },
    }
}
