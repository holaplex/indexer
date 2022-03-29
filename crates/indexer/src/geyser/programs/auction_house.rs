use anchor_lang::AccountDeserialize;
use mpl_auction_house::{
    receipt::{
        BidReceipt, ListingReceipt, PurchaseReceipt, BID_RECEIPT_SIZE, LISTING_RECEIPT_SIZE,
        PURCHASE_RECEIPT_SIZE,
    },
    AuctionHouse, AUCTION_HOUSE_SIZE,
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

async fn process_listing_receipt(client: &Client, update: AccountUpdate) -> Result<()> {
    let listing_receipt: ListingReceipt =
        ListingReceipt::try_deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize listing receipt data")?;

    receipt::process_listing_receipt(client, update.key, listing_receipt).await
}
async fn process_bid_receipt(client: &Client, update: AccountUpdate) -> Result<()> {
    let bid_receipt: BidReceipt = BidReceipt::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize bid receipt data")?;

    receipt::process_bid_receipt(client, update.key, bid_receipt).await
}
async fn process_purchase_receipt(client: &Client, update: AccountUpdate) -> Result<()> {
    let purchase_receipt: PurchaseReceipt =
        PurchaseReceipt::try_deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize purchase receipt data")?;

    receipt::process_purchase_receipt(client, update.key, purchase_receipt).await
}
pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        AUCTION_HOUSE_SIZE => process_auction_house(client, update).await,
        LISTING_RECEIPT_SIZE => process_listing_receipt(client, update).await,
        BID_RECEIPT_SIZE => process_bid_receipt(client, update).await,
        PURCHASE_RECEIPT_SIZE => process_purchase_receipt(client, update).await,
        _ => Ok(()),
    }
}
