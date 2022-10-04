use hpl_reward_center::state::{PayoutOperation, RewardCenter};
use indexer_core::{
    db::{
        custom_types::PayoutOperationEnum,
        insert_into,
        models::{RewardCenter as DbRewardCenter, RewardRule as DbRewardRule},
        tables::{reward_centers, reward_rules},
    },
    prelude::*,
};

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: RewardCenter,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardCenter {
        address: Owned(bs58::encode(key).into_string()),
        token_mint: Owned(bs58::encode(account_data.token_mint).into_string()),
        auction_house: Owned(bs58::encode(account_data.auction_house).into_string()),
        bump: account_data.bump.into(),
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
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

    let rules = account_data.reward_rules;

    let values = DbRewardRule {
        reward_center_address: Owned(key.to_string()),
        seller_reward_payout_basis_points: rules.seller_reward_payout_basis_points.try_into()?,
        mathematical_operand: match rules.mathematical_operand {
            PayoutOperation::Multiple => PayoutOperationEnum::Multiple,
            PayoutOperation::Divide => PayoutOperationEnum::Divide,
        },
        payout_numeral: rules.payout_numeral.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(reward_rules::table)
                .values(&values)
                .on_conflict(reward_rules::reward_center_address)
                .do_update()
                .set(&values)
                .execute(db)
        })
        .await
        .context("Failed to insert reward center rules")?;

    Ok(())
}
