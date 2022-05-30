use anchor_lang_v0_20::AccountDeserialize;
use mpl_auction_house::{
    receipt::{
        BidReceipt, ListingReceipt, PurchaseReceipt, BID_RECEIPT_SIZE, LISTING_RECEIPT_SIZE,
        PURCHASE_RECEIPT_SIZE,
    },
    AuctionHouse, AUCTION_HOUSE_SIZE,
};

use super::{
    accounts::{auction_house, receipt},
    instructions::{
        buy, cancel, deposit, execute_sale, public_buy, sell, withdraw, withdraw_from_fee,
        withdraw_from_treasury,
    },
    AccountUpdate, Client,
};
use crate::prelude::*;

// Anchor Discriminators
const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const PUBLIC_BUY: [u8; 8] = [169, 84, 218, 35, 42, 206, 16, 171];
const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const EXECUTE_SALE: [u8; 8] = [37, 74, 217, 157, 79, 49, 35, 6];
const CANCEL: [u8; 8] = [232, 219, 223, 41, 219, 236, 220, 190];
const DEPOSIT: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
const WITHDRAW: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
const WITHDRAW_FROM_FEE: [u8; 8] = [179, 208, 190, 154, 32, 179, 19, 59];
const WITHDRAW_FROM_TREASURY: [u8; 8] = [0, 164, 86, 76, 56, 72, 12, 170];

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

pub(crate) async fn process_instruction(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
) -> Result<()> {
    let discriminator: [u8; 8] = data[..8].try_into()?;
    let params = data[8..].to_vec();

    match discriminator {
        BUY => buy::process(client, &params, accounts).await,
        PUBLIC_BUY => public_buy::process(client, &params, accounts).await,
        SELL => sell::process(client, &params, accounts).await,
        EXECUTE_SALE => execute_sale::process(client, &params, accounts).await,
        CANCEL => cancel::process(client, &params, accounts).await,
        DEPOSIT => deposit::process(client, &params, accounts).await,
        WITHDRAW => withdraw::process(client, &params, accounts).await,
        WITHDRAW_FROM_FEE => withdraw_from_fee::process(client, &params, accounts).await,
        WITHDRAW_FROM_TREASURY => withdraw_from_treasury::process(client, &params, accounts).await,
        _ => Ok(()),
    }
}
