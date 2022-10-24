use borsh::BorshDeserialize;
use hpl_reward_center::instruction::CloseListing;
use indexer_core::db::{
    insert_into,
    models::CancelInstruction,
    select,
    tables::{cancel_instructions, listings, rewards_listings},
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
    let listing_address = accts[1];
    let trade_state = accts[9];
    let closed_at = Some(Utc::now().naive_utc());

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
                .execute(db);

                update(
                  listings::table.filter(listings::trade_state.eq(trade_state)),
              )
              .set((
                  listings::canceled_at.eq(closed_at),
                  listings::slot.eq(slot),
              ))
              .execute(db);

              Ok(())
            })
        })
        .await
        .context("failed to update rewards listing closed at or general listing canceled at")?;

    Ok(())
}
