use anchor_lang::AccountDeserialize;
use mpl_auction_house::AuctionHouse;

use super::{accounts::auction_house, AccountUpdate, Client};
use crate::prelude::*;

async fn process_auction_house(client: &Client, update: AccountUpdate) -> Result<()> {
    let house: AuctionHouse = AuctionHouse::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize auction house data")?;

    auction_house::process(client, update.key, house).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if update.data.len() == 1 {
        // Ignore trade PDA
        return Ok(());
    }

    // TODO: add additional account types here
    process_auction_house(client, update).await
}
