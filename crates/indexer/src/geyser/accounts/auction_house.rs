use indexer_core::{
    db::{insert_into, models::AuctionHouse as DbAuctionHouse, tables::auction_houses},
    prelude::*,
};
use mpl_auction_house::AuctionHouse;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: AuctionHouse,
) -> Result<()> {
    let row = DbAuctionHouse {
        address: Owned(bs58::encode(key).into_string()),
        treasury_mint: Owned(bs58::encode(account_data.treasury_mint).into_string()),
        auction_house_treasury: Owned(
            bs58::encode(account_data.auction_house_treasury).into_string(),
        ),
        treasury_withdrawal_destination: Owned(
            bs58::encode(account_data.treasury_withdrawal_destination).into_string(),
        ),
        fee_withdrawal_destination: Owned(
            bs58::encode(account_data.fee_withdrawal_destination).into_string(),
        ),
        authority: Owned(bs58::encode(account_data.authority).into_string()),
        creator: Owned(bs58::encode(account_data.creator).into_string()),
        bump: account_data.bump.into(),
        treasury_bump: account_data.treasury_bump.into(),
        fee_payer_bump: account_data.fee_payer_bump.into(),
        seller_fee_basis_points: account_data
            .seller_fee_basis_points
            .try_into()
            .context("Seller fee basis points is too big to store")?,
        requires_sign_off: account_data.requires_sign_off,
        can_change_sale_price: account_data.can_change_sale_price,
        auction_house_fee_account: Owned(
            bs58::encode(account_data.auction_house_fee_account).into_string(),
        ),
    };

    client
        .db()
        .run(move |db| {
            insert_into(auction_houses::table)
                .values(&row)
                .on_conflict(auction_houses::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert auction house")?;

    Ok(())
}
