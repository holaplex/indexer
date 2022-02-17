use indexer_core::{
    db::{insert_into, models::Receipt as DbReceipt, tables::receipts},
    prelude::*,
};
use mpl_auction_house::Receipt;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: AuctionHouse,
) -> Result<()> {
    let row = DbReceipt {
        address: Owned(bs58::encode(key).into_string()),
        trade_state: Owned(bs58::encode(account_data.trade_state).into_string()),
        bookkeeper: Owned(bs58::encode(account_data.bookkeeper).into_string()),
        auction_house: Owned(bs58::encode(account_data.auction_house).into_string()),
        wallet: Owned(bs58::encode(account_data.wallet).into_string()),
        token_account: Owned(bs58::encode(account_data.token_account).into_string()),
        metadata_mint: Owned(bs58::encode(account_data.metadata_mint).into_string()),
        price: account_data.price.into(),
        token_size: account_data.token_size.into(),
        bump: account_data.bump.into(),
        trade_state_bump: account_data.trade_state_bump.into(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(receipts::table)
                .values(&row)
                .on_conflict(receipts::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert auction house receipt")?;

    Ok(())
}
