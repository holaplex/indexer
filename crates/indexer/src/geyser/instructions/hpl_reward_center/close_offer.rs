use borsh::BorshDeserialize;
use hpl_reward_center::offers::close::CloseOfferParams;
use indexer_core::db::{
    insert_into,
    models::HplRewardCenterCloseoffer,
    tables::{hpl_reward_center_close_offer_ins, offers, rewards_offers},
    update,
};
use solana_program::pubkey::Pubkey;

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params =
        CloseOfferParams::try_from_slice(data).context("failed to deserialize close offer args")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();
    let offer_address = accts[1].clone();
    let trade_state = accts[12].clone();
    let closed_at = Some(Utc::now().naive_utc());
    let slot: i64 = slot.try_into()?;
    let escrow_payment_bump = params.escrow_payment_bump.try_into()?;

    client
        .db()
        .run({
            let offer_address = offer_address.clone();
            move |db| {
                let (token_size, buyer_price): (i64, i64) = rewards_offers::table
                    .select((rewards_offers::token_size, rewards_offers::price))
                    .filter(rewards_offers::address.eq(offer_address))
                    .first(db)?;

                let row = HplRewardCenterCloseoffer {
                    wallet: Owned(accts[0].clone()),
                    offer: Owned(accts[1].clone()),
                    treasury_mint: Owned(accts[2].clone()),
                    token_account: Owned(accts[3].clone()),
                    receipt_account: Owned(accts[4].clone()),
                    escrow_payment_account: Owned(accts[5].clone()),
                    metadata: Owned(accts[6].clone()),
                    token_mint: Owned(accts[7].clone()),
                    authority: Owned(accts[8].clone()),
                    reward_center: Owned(accts[9].clone()),
                    auction_house: Owned(accts[10].clone()),
                    auction_house_fee_account: Owned(accts[11].clone()),
                    trade_state: Owned(accts[12].clone()),
                    ah_auctioneer_pda: Owned(accts[13].clone()),
                    escrow_payment_bump,
                    buyer_price,
                    token_size,
                    created_at: Utc::now().naive_utc(),
                    slot,
                };

                insert_into(hpl_reward_center_close_offer_ins::table)
                    .values(&row)
                    .execute(db)
            }
        })
        .await
        .context("failed to insert reward center close offer instruction ")?;

    client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                update(rewards_offers::table.filter(rewards_offers::address.eq(offer_address)))
                    .set((
                        rewards_offers::closed_at.eq(closed_at),
                        rewards_offers::slot.eq(slot),
                    ))
                    .execute(db)?;

                update(offers::table.filter(offers::trade_state.eq(trade_state)))
                    .set((offers::canceled_at.eq(closed_at), offers::slot.eq(slot)))
                    .execute(db)
            })
        })
        .await
        .context("failed to update rewards offer closed at or general offer canceled at")?;

    Ok(())
}
