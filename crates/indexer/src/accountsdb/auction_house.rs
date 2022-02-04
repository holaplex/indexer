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
    // let x = vec![
    //     40, 108, 215, 107, 213, 85, 245, 48, 195, 140, 128, 103, 133, 144, 25, 42, 197, 98, 54,
    //     237, 222, 28, 212, 14, 168, 254, 113, 11, 66, 153, 83, 234, 124, 179, 127, 134, 133, 222,
    //     68, 185, 101, 177, 116, 158, 169, 3, 144, 38, 254, 95, 35, 21, 151, 21, 171, 117, 187, 249,
    //     108, 32, 125, 76, 153, 35, 176, 2, 207, 67, 96, 97, 28, 49, 244, 16, 196, 38, 22, 192, 95,
    //     16, 41, 59, 153, 127, 134, 15, 176, 56, 96, 187, 3, 11, 214, 108, 52, 57, 143, 162, 101,
    //     233, 249, 185, 101, 230, 45, 137, 58, 91, 227, 47, 75, 154, 16, 120, 62, 90, 13, 120, 1,
    //     254, 75, 85, 206, 53, 100, 2, 146, 58, 144, 10, 36, 233, 56, 143, 245, 202, 6, 155, 136,
    //     87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196, 57, 220, 26, 235, 59,
    //     85, 152, 160, 240, 0, 0, 0, 0, 1, 244, 16, 196, 38, 22, 192, 95, 16, 41, 59, 153, 127, 134,
    //     15, 176, 56, 96, 187, 3, 11, 214, 108, 52, 57, 143, 162, 101, 233, 249, 185, 101, 230, 244,
    //     16, 196, 38, 22, 192, 95, 16, 41, 59, 153, 127, 134, 15, 176, 56, 96, 187, 3, 11, 214, 108,
    //     52, 57, 143, 162, 101, 233, 249, 185, 101, 230, 255, 251, 255, 244, 1, 1, 1, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //     0, 0, 0, 0, 0,
    // ];

    // let decoded = bs58::decode("Ccp4r8JVBK9vEJW6FyhfyUktuygNQu9XoCavf3EShx8j").into_vec().unwrap();

    println!("auction_house::process called!");
    let account_data: AuctionHouse = AuctionHouse::try_deserialize(&mut data.as_slice())
        .context("failed to deserialize the auction house data!")?;
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
