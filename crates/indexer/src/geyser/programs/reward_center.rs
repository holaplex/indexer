use anchor_lang_v0_24::AccountDeserialize;
use mpl_reward_center::{Listing, Offer, RewardCenter, PurchaseTicket};

use super::{accounts::reward_center, AccountUpdate, Client};
use crate::prelude::*;

pub const REWARD_CENTER_SIZE: usize = RewardCenter::size();
pub const LISTING_SIZE: usize = Listing::size();
pub const OFFER_SIZE: usize = Offer::size();
pub const PURCHASE_TICKET_SIZE: usize = PurchaseTicket::size();

async fn process_reward_center(client: &Client, update: AccountUpdate) -> Result<()> {
    let reward_center: RewardCenter = RewardCenter::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize reward center data")?;

    reward_center::reward_center::process(client, update.key, reward_center, update.slot, update.write_version).await
}

async fn process_listing(client: &Client, update: AccountUpdate) -> Result<()> {
    let listing: Listing = Listing::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize listing data")?;

    reward_center::listing::process(client, update.key, listing, update.slot, update.write_version).await
}

async fn process_offer(client: &Client, update: AccountUpdate) -> Result<()> {
    let offer: Offer = Offer::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize offer data")?;

    reward_center::offer::process(client, update.key, offer, update.slot, update.write_version).await
}

async fn process_purchase_ticket(client: &Client, update: AccountUpdate) -> Result<()> {
    let purchase_ticket: PurchaseTicket = PurchaseTicket::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize purchase ticket data")?;

    reward_center::purchase_ticket::process(client, update.key, purchase_ticket, update.slot, update.write_version).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        REWARD_CENTER_SIZE => process_reward_center(client, update).await,
        LISTING_SIZE => process_listing(client, update).await,
        OFFER_SIZE => process_offer(client, update).await,
        PURCHASE_TICKET_SIZE => process_purchase_ticket(client, update).await,
        _ => Ok(()),
    }
}
