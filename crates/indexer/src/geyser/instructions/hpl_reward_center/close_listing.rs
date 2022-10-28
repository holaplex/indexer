use indexer_core::db::{
    insert_into,
    models::HplRewardCenterCloseListing,
    tables::{hpl_reward_center_close_listing_ins, listings, rewards_listings},
    update,
};
use solana_program::pubkey::Pubkey;

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    _data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();
    let listing_address = accts[1].clone();
    let trade_state = accts[9].clone();
    let closed_at = Some(Utc::now().naive_utc());
    let slot: i64 = slot.try_into()?;

    client
        .db()
        .run(move |db| {
            let token_size = rewards_listings::table
                .select(rewards_listings::token_size)
                .filter(rewards_listings::address.eq(accts[1].clone()))
                .first(db)
                .optional()?;

            let row = HplRewardCenterCloseListing {
                wallet: Owned(accts[0].clone()),
                listing: Owned(accts[1].clone()),
                metadata: Owned(accts[2].clone()),
                token_account: Owned(accts[3].clone()),
                token_mint: Owned(accts[4].clone()),
                authority: Owned(accts[5].clone()),
                reward_center: Owned(accts[6].clone()),
                auction_house: Owned(accts[7].clone()),
                auction_house_fee_account: Owned(accts[8].clone()),
                trade_state: Owned(accts[9].clone()),
                ah_auctioneer_pda: Owned(accts[10].clone()),
                token_size: token_size.unwrap_or_default(),
                created_at: Utc::now().naive_utc(),
                slot,
            };

            insert_into(hpl_reward_center_close_listing_ins::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert reward center close listing instruction ")?;

    client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                update(
                    rewards_listings::table.filter(rewards_listings::address.eq(listing_address)),
                )
                .set((
                    rewards_listings::closed_at.eq(closed_at),
                    rewards_listings::slot.eq(slot),
                ))
                .execute(db)?;

                update(listings::table.filter(listings::trade_state.eq(trade_state)))
                    .set((listings::canceled_at.eq(closed_at), listings::slot.eq(slot)))
                    .execute(db)
            })
        })
        .await
        .context("failed to update rewards listing closed at or general listing canceled at")?;

    Ok(())
}
