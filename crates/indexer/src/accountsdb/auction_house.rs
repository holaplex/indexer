use anchor_lang::AccountDeserialize;
use indexer_core::{
    db::{insert_into, models::AuctionHouse as DbAuctionHouse, tables::auction_houses},
    prelude::*,
    pubkeys,
};
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auction_house_fee_account_address,
        find_auction_house_treasury_address,
    },
    AuctionHouse,
};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{client::prelude::*, prelude::*, util, Client};

pub async fn process(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    let account_data: AuctionHouse = AuctionHouse::try_deserialize(&mut data.as_slice())
        .context("failed to deserialize the auction house data!")?;
    dbg!("{:?}", account_data.treasury_mint);
    let (ah_address, _) =
        find_auction_house_address(&account_data.authority, &account_data.treasury_mint);
    let (ah_fee_acc_addr, _) = find_auction_house_fee_account_address(&key);
    let (ah_treasury_addr, _) = find_auction_house_treasury_address(&key);

    if ah_address != key
        || ah_fee_acc_addr != account_data.auction_house_fee_account
        || ah_treasury_addr != account_data.auction_house_treasury
    {
        println!("keys mismatch");
        return Ok(());
    }
    println!("key match!");
    let row = DbAuctionHouse {
        address: Owned(bs58::encode(key).into_string()),
        treasury_mint: Owned(bs58::encode(account_data.treasury_mint).into_string()),
        auction_house_treasury: Owned(
            bs58::encode(account_data.auction_house_treasury).into_string(),
        ),
        treasury_withdrawal_destination: Owned(
            bs58::encode(account_data.auction_house_treasury).into_string(),
        ),
        fee_withdrawal_destination: Owned(
            bs58::encode(account_data.fee_withdrawal_destination).into_string(),
        ),
        authority: Owned(bs58::encode(account_data.authority).into_string()),
        creator: Owned(bs58::encode(account_data.creator).into_string()),
        bump: i16::from(account_data.bump),
        treasury_bump: i16::from(account_data.treasury_bump),
        fee_payer_bump: i16::from(account_data.fee_payer_bump),
        seller_fee_basis_points: account_data.seller_fee_basis_points as i16,
        requires_sign_off: account_data.requires_sign_off,
        can_change_sale_price: account_data.can_change_sale_price,
    };
    client
        .db(move |db| {
            insert_into(auction_houses::table)
                .values(&row)
                .on_conflict(auction_houses::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert auction_house!")?;
    println!("inserted into auction_houses table!");
    Ok(())
}
