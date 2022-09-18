use indexer_core::{
    db::{
        insert_into,
        models::{RewardCenter as DbRewardCenter, RewardRules as DbRewardRules},
        tables::{reward_centers, reward_ruless},
    },
    prelude::*,
};
use mpl_reward_center::RewardCenter;

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

    let reward_rules_row = DbRewardRule {
        reward_center_address: Owned(bs58::encode(key).into_string()),
        seller_reward_payout_basis_points: account_data
            .reward_rules
            .seller_reward_payout_basis_points
            .into(),
        payout_divider: account_data.reward_rules.payout_divider.into(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(reward_centers::table)
                .values(&row)
                .on_conflict(reward_centers::address)
                .do_update()
                .set(&row)
                .execute(db);

            insert_into(reward_ruless::table)
                .values(&reward_rules_row)
                .on_conflict(reward_ruless::reward_center_address)
                .do_update()
                .set(&reward_rules_row)
                .execute(db)
        })
        .await
        .context("Failed to insert reward center")?;
}
