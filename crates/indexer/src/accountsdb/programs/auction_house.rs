use anchor_lang::AccountDeserialize;
use mpl_auction_house::{
    AuctionHouse, Receipt, AUCTION_HOUSE_SIZE, RECEIPT_SIZE, TRADE_STATE_SIZE,
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

async fn process_receipt(client: &Client, update: AccountUpdate) -> Result<()> {
    let receipt: Receipt = Receipt::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize auction house receipt")?;

    receipt::process(client, update.key, house).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        AUCTION_HOUSE_SIZE => process_auction_house(client, update).await,
        RECEIPT_SIZE => process_receipt(client, update).await,
        TRADE_STATE_SIZE => Ok(()),
    }

    // TODO: add additional account types here
    process_auction_house(client, update).await
}
