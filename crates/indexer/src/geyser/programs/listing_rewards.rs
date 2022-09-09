use anchor_lang_v0_24::AccountDeserialize;
use mpl_listing_rewards::{Listing, Offer, RewardCenter};

use super::{accounts::reward_center, AccountUpdate, Client};
use crate::prelude::*;

pub const REWARD_CENTER_SIZE: usize = RewardCenter::size();
pub const LISTING_SIZE: usize = Listing::size();
pub const OFFER_SIZE: usize = Offer::size();

async fn process_reward_center(client: &Client, update: AccountUpdate) -> Result<()> {
    let reward_center: RewardCenter = RewardCenter::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize reward center data")?;

    listing_rewards::reward_center::process(client, update.key, reward_center).await
}

async fn process_listing(client: &Client, update: AccountUpdate) -> Result<()> {
    let listing: Listing = Listing::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize listing data")?;

    listing_rewards::listing::process(client, update.key, listing).await
}

async fn process_offer(client: &Client, update: AccountUpdate) -> Result<()> {
    let offer: Offer = Offer::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize offer data")?;

    listing_rewards::offer::process(client, update.key, offer).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        REWARD_CENTER_SIZE => process_reward_center(client, update).await,
        LISTING_SIZE => process_listing(client, update).await,
        OFFER_SIZE => process_offer(client, update).await,
        _ => Ok(()),
    }
}
