use borsh::BorshDeserialize;
use indexer_core::db::{models::BuyInstruction, insert_into, tables::buy_instructions};

use super::instructions::buy::upsert_into_offers_table;


use super::Client;
use crate::prelude::*;

const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const PUBLIC_BUY: [u8; 8] = [169, 84, 218, 35, 42, 206, 16, 171];
const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const EXECUTE_SALE: [u8; 8] = [37, 74, 217, 157, 79, 49, 35, 6];
const CANCEL: [u8; 8] = [232, 219, 223, 41, 219, 236, 220, 190];
const DEPOSIT: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
const WITHDRAW: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
const WITHDRAW_FROM_FEE: [u8; 8] = [179, 208, 190, 154, 32, 179, 19, 59];
const WITHDRAW_FROM_TREASURY: [u8; 8] = [0, 164, 86, 76, 56, 72, 12, 170];

#[derive(BorshDeserialize, Debug, Clone)]
struct MEBuyInstructionData {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
    expiry: u64,
}

pub(crate) async fn process_buy(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = MEBuyInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME buy instructions")?;

    // ME2 accounts
    if accounts.len() != 12 {
        debug!("invalid accounts for BuyInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = BuyInstruction {
        wallet: Owned(accts[0].clone()),
        payment_account: Owned(accts[0].clone()),
        transfer_authority: Owned(accts[1].clone()),
        treasury_mint: Borrowed("So11111111111111111111111111111111111111112"),
        token_account: Owned(accts[2].clone()),
        metadata: Owned(accts[3].clone()),
        escrow_payment_account: Owned(accts[4].clone()),
        authority: Owned(accts[5].clone()),
        auction_house: Owned(accts[6].clone()),
        auction_house_fee_account: Owned(accts[7].clone()),
        // TODO: make this optional
        buyer_trade_state: Borrowed("magice_eden_haus"), //
        trade_state_bump: params.trade_state_bump.try_into()?,
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    upsert_into_offers_table(client, row.clone())
        .await
        .context("failed to insert offer")?;

    client
        .db()
        .run(move |db| {
            insert_into(buy_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert buy instruction ")?;
    Ok(())
}

pub(crate) async fn process_instruction(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let discriminator: [u8; 8] = data[..8].try_into()?;
    let params = data[8..].to_vec();

    debug!("discriminator: {:?}", discriminator);

    match discriminator {
        BUY => process_buy(client, &params, accounts, slot).await,
        // PUBLIC_BUY => public_buy::process(client, &params, accounts, slot).await,
        // SELL => sell::process(client, &params, accounts, slot).await,
        // EXECUTE_SALE => execute_sale::process(client, &params, accounts, slot).await,
        // CANCEL => cancel::process(client, &params, accounts, slot).await,
        // DEPOSIT => deposit::process(client, &params, accounts, slot).await,
        // WITHDRAW => withdraw::process(client, &params, accounts, slot).await,
        // WITHDRAW_FROM_FEE => withdraw_from_fee::process(client, &params, accounts, slot).await,
        // WITHDRAW_FROM_TREASURY => {
        //     withdraw_from_treasury::process(client, &params, accounts, slot).await
        // },
        _ => Ok(()),
    }
}
