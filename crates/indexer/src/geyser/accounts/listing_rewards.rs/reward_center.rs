use indexer_core::{
    db::{insert_into, models::RewardCenter as DbRewardCenter, tables::reward_centers},
    prelude::*,
};
use mpl_listing_rewards::RewardCenter;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: RewardCenter,
) -> Result<()> {
    let row = DbRewardCenter {
        address: Owned(bs58::encode(key).into_string()),
        token_mint: Owned(bs58::encode(account_data.token_mint).into_string()),
        auction_house: Owned(bs58::encode(account_data.auction_house).into_string()),
        bump: account_data.bump.into(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(reward_centers::table)
                .values(&row)
                .on_conflict(reward_centers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert reward center")?;
}
