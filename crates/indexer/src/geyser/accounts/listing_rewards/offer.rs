use indexer_core::{
    db::{insert_into, models::RewardsOffers as DbRewardsOffers, tables::rewards_offers},
    prelude::*,
};
use mpl_listing_rewards::Offer;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, key: Pubkey, account_data: Listing) -> Result<()> {
    let row = DbRewardsOffers {
        address: Owned(bs58::encode(key).into_string()),
        is_initialized: account_data.is_initialized,
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        buyer: Owned(bs58::encode(account_data.buyer).into_string()),
        metadata: Owned(bs58::encode(account_data.metadata).into_string()),
        price: account_data
            .price
            .try_into()
            .context("Price is too big to store"),
        token_size: account_data
            .token_size
            .try_into()
            .context("Token size is too big to store"),
        created_at: account_data
            .created_at
            .try_into()
            .context("Created at is too big to store"),
        canceled_at: account_data.canceled_at,
        purchased_at: account.data.purchased_at,
        reward_redeemed_at: account.data.reward_redeemed_at,
    };

    client
        .db()
        .run(move |db| {
            insert_into(rewards_offers::table)
                .values(&row)
                .on_conflict(rewards_offers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert rewards offer")?;
}
