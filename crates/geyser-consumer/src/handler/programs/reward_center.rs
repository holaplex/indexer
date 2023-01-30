use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use hpl_reward_center::state::{Listing, Offer, RewardCenter};
use indexer::prelude::*;

use super::{
    accounts::hpl_reward_center as reward_center,
    instructions::hpl_reward_center::{accept_offer, buy_listing, close_listing, close_offer},
    magic_eden_haus::get_block_time,
    AccountUpdate, Client,
};

const ACCEPT_OFFER: [u8; 8] = [227, 82, 234, 131, 1, 18, 48, 2];
const BUY_LISTING: [u8; 8] = [115, 149, 42, 108, 44, 49, 140, 153];
const CLOSE_OFFER: [u8; 8] = [191, 72, 67, 35, 239, 209, 97, 132];
const CLOSE_LISTING: [u8; 8] = [33, 15, 192, 81, 78, 175, 159, 97];

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

async fn process_listing(
    client: &Client,
    update: AccountUpdate,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let listing: Listing = Listing::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize listing data")?;

    reward_center::listing::process(
        client,
        update.key,
        listing,
        update.slot,
        update.write_version,
        timestamp,
    )
    .await
}

async fn process_offer(
    client: &Client,
    update: AccountUpdate,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let offer: Offer = Offer::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize offer data")?;

    reward_center::offer::process(
        client,
        update.key,
        offer,
        update.slot,
        update.write_version,
        timestamp,
    )
    .await
}

pub(crate) async fn process_instruction(
    client: &Client,
    tx_signature: Vec<u8>,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let discriminator: [u8; 8] = data[..8].try_into()?;
    let params = data[8..].to_vec();
    let signature = solana_sdk::signature::Signature::new(&tx_signature).to_string();

    let timestamp = get_block_time(client, slot)?;

    match discriminator {
        ACCEPT_OFFER => {
            accept_offer::process(client, signature, &params, accounts, slot, timestamp).await
        },
        BUY_LISTING => {
            buy_listing::process(client, signature, &params, accounts, slot, timestamp).await
        },
        CLOSE_OFFER => close_offer::process(client, &params, accounts, slot, timestamp).await,
        CLOSE_LISTING => close_listing::process(client, &params, accounts, slot, timestamp).await,
        _ => Ok(()),
    }
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let discrim: [u8; 8] = update.data[..8].try_into()?;

    let timestamp = get_block_time(client, update.slot)?;

    match discrim {
        d if d == RewardCenter::discriminator() => process_reward_center(client, update).await,
        d if d == Listing::discriminator() => process_listing(client, update, timestamp).await,
        d if d == Offer::discriminator() => process_offer(client, update, timestamp).await,
        _ => Ok(()),
    }
}
