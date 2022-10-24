use borsh::BorshDeserialize;
use hpl_reward_center::instruction::CloseListing;
use indexer_core::db::{
    insert_into,
    models::CancelInstruction,
    select,
    tables::{cancel_instructions, offers, rewards_offers},
    update,
};
use mpl_auction_house::pda::find_auctioneer_trade_state_address;
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
        CloseListing::try_from_slice(data).context("failed to deserialize close listing args")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();
    let offer_address = accts[1].clone();
    let trade_state = accts[12].clone();
    let closed_at = Some(Utc::now().naive_utc());
    let slot: i64 = slot.try_into()?;

    client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                update(rewards_offers::table.filter(rewards_offers::address.eq(offer_address)))
                    .set((
                        rewards_offers::closed_at.eq(closed_at),
                        rewards_offers::slot.eq(slot),
                    ))
                    .execute(db);

                update(offers::table.filter(offers::trade_state.eq(trade_state)))
                    .set((offers::canceled_at.eq(closed_at), offers::slot.eq(slot)))
                    .execute(db);

                Result::<_>::Ok(())
            })
        })
        .await
        .context("failed to update rewards offer closed at or general offer canceled at")?;

    Ok(())
}
