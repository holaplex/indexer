use anchor_lang::AccountDeserialize;
use indexer_core::{
    db::{insert_into, models::AuctionHouse as DbAuctionHouse, tables::auction_houses},
    prelude::*,
    pubkeys,
};
use mpl_auction_house::{pda::find_auction_house_address, AuctionHouse, AUCTION_HOUSE_SIZE};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{client::prelude::*, prelude::*, util, Client};

pub fn process(client: &Client) -> Result<()> {
    let db = client.db()?;

    // Uses Solana JSON api to return all auction houses accounts with data in base64 format
    let res = client.get_program_accounts(pubkeys::auction_house(), RpcProgramAccountsConfig {
        account_config: RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        filters: Some(vec![RpcFilterType::DataSize(
            AUCTION_HOUSE_SIZE.try_into().unwrap(),
        )]),
        ..RpcProgramAccountsConfig::default()
    });

    // parses each auction house account data and inserts it to auction houses table
    res.context("Failed to get accounts!")?
        .into_iter()
        .filter_map(|(key, mut acc)| {
            let account_data: AuctionHouse =
                deserialize_account_data(&util::account_as_info(&key, false, false, &mut acc))
                    .map_err(|e| debug!("Failed to parse account data! {:?}", e))
                    .ok()?;
            let (auction_house_address, _bump) =
                find_auction_house_address(&account_data.authority, &account_data.treasury_mint);
            if auction_house_address != key {
                println!("Auction house keys mismatch!");
                return None;
            }
            Some((key, account_data))
        })
        .for_each(|acc| {
            let row = DbAuctionHouse {
                auction_house_address: Owned(acc.0.to_string()),
                treasury_mint: Owned(bs58::encode(acc.1.treasury_mint).into_string()),
                auction_house_treasury: Owned(
                    bs58::encode(acc.1.auction_house_treasury).into_string(),
                ),
                treasury_withdrawal_destination: Owned(
                    bs58::encode(acc.1.auction_house_treasury).into_string(),
                ),
                fee_withdrawal_destination: Owned(
                    bs58::encode(acc.1.fee_withdrawal_destination).into_string(),
                ),
                authority: Owned(bs58::encode(acc.1.authority).into_string()),
                creator: Owned(bs58::encode(acc.1.creator).into_string()),
                bump: i16::from(acc.1.bump),
                treasury_bump: i16::from(acc.1.treasury_bump),
                fee_payer_bump: i16::from(acc.1.fee_payer_bump),
                seller_fee_basis_points: acc.1.seller_fee_basis_points as i16,
                requires_sign_off: acc.1.requires_sign_off,
                can_change_sale_price: acc.1.can_change_sale_price,
            };
            insert_into(auction_houses::table)
                .values(&row)
                .on_conflict(auction_houses::auction_house_address)
                .do_update()
                .set(&row)
                .execute(&db)
                .map_err(|e| debug!("Failed to insert auction house into database! {:?}", e))
                .ok();
        });
    Ok(())
}

// deserializes auction house acconnt data
pub fn deserialize_account_data(a: &AccountInfo) -> Result<AuctionHouse, ProgramError> {
    Ok(AuctionHouse::try_deserialize(&mut a.data.borrow_mut().as_ref()).unwrap())
}
