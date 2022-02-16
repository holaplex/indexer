use anchor_lang::AccountDeserialize;
use mpl_auction_house::AuctionHouse;

use super::accounts::auction_house;
use crate::{prelude::*, Client};

async fn process_auction_house(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    if data.len() == 1 {
        return Ok(());
    }

    let house: AuctionHouse = AuctionHouse::try_deserialize(&mut data.as_slice())
        .context("Failed to deserialize auction house data")?;

    auction_house::process(client, key, house).await
}

pub(crate) async fn process(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    // TODO: add additional account types here
    process_auction_house(client, key, data).await
}
