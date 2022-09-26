use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use mpl_reward_center::state::{Listing, Offer, PurchaseTicket, RewardCenter};

use super::{accounts::mpl_reward_center as reward_center, AccountUpdate, Client};
use crate::prelude::*;

async fn process_reward_center(client: &Client, update: AccountUpdate) -> Result<()> {
    let reward_center: RewardCenter = RewardCenter::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize reward center data")?;

    reward_center::reward_center::process(
        client,
        update.key,
        reward_center,
        update.slot,
        update.write_version,
    )
    .await
}

async fn process_listing(client: &Client, update: AccountUpdate) -> Result<()> {
    let listing: Listing = Listing::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize listing data")?;

    reward_center::listing::process(
        client,
        update.key,
        listing,
        update.slot,
        update.write_version,
    )
    .await
}

async fn process_offer(client: &Client, update: AccountUpdate) -> Result<()> {
    let offer: Offer = Offer::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize offer data")?;

    reward_center::offer::process(client, update.key, offer, update.slot, update.write_version)
        .await
}

async fn process_purchase_ticket(client: &Client, update: AccountUpdate) -> Result<()> {
    let purchase_ticket: PurchaseTicket =
        PurchaseTicket::try_deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize purchase ticket data")?;

    reward_center::purchase_ticket::process(
        client,
        update.key,
        purchase_ticket,
        update.slot,
        update.write_version,
    )
    .await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let discrim: [u8; 8] = update.data[..8].try_into()?;

    match discrim {
        d if d == RewardCenter::discriminator() => process_reward_center(client, update).await,
        d if d == Listing::discriminator() => process_listing(client, update).await,
        d if d == Offer::discriminator() => process_offer(client, update).await,
        d if d == PurchaseTicket::discriminator() => process_purchase_ticket(client, update).await,
        _ => Ok(()),
    }
}
