use anchor_lang_v0_24::AccountDeserialize;

use super::{
    AccountUpdate, Client,
};
use crate::prelude::*;

async fn process_reward_center(client: &Client, update: AccountUpdate) -> Result<()> {
    let house: AuctionHouse = AuctionHouse::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize auction house data")?;

    auction_house::process(client, update.key, house).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let account_discriminator = &update.data[..8];

    match account_discriminator {
         => process_auction_house(client, update).await,
        _ => Ok(()),
    }
}
