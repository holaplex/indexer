use indexer_core::{
    db::{
        insert_into,
        models::RewardCenterPurchaseTicket as DbRewardCenterPurchaseTicketPurchaseTicket,
        tables::reward_center_purchase_tickets,
    },
    prelude::*,
};
use mpl_reward_center::PurchaseTicket;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: PurchaseTicket,
) -> Result<()> {
    let row = DbRewardCenterPurchaseTicket {
        address: Owned(bs58::encode(key).into_string()),
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        buyer: Owned(bs58::encode(account_data.buyer).into_string()),
        seller: Owned(bs58::encode(account_data.seller).into_string()),
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
    };

    client
        .db()
        .run(move |db| {
            insert_into(reward_center_purchase_tickets::table)
                .values(&row)
                .on_conflict(reward_center_purchase_tickets::address)
                .do_update()
                .set(&row)
                .execute(db);
        })
        .await
        .context("Failed to insert purchase ticket")?;
}
