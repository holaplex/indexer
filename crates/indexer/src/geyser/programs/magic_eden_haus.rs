use borsh::BorshDeserialize;
use indexer_core::db::{
    insert_into,
    models::{BuyInstruction, SellInstruction, ExecuteSaleInstruction},
    tables::{buy_instructions, sell_instructions, execute_sale_instructions},
};

use indexer_core::pubkeys;

use super::{
    instructions::buy::upsert_into_offers_table, 
    instructions::sell::upsert_into_listings_table, 
    instructions::execute_sale::upsert_into_purchases_table, 
    Client
};
use crate::prelude::*;

const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const EXECUTE_SALE: [u8; 8] = [37, 74, 217, 157, 79, 49, 35, 6];

#[derive(BorshDeserialize, Debug, Clone)]
struct MEInstructionData {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
    expiry: u64,
}

pub(crate) async fn process_execute_sale(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME instructions")?;

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = ExecuteSaleInstruction {
        buyer: Owned(accts[0].clone()),
        seller: Owned(accts[1].clone()),
        token_account: Owned(accts[7].clone()),
        token_mint: Owned(accts[4].clone()),
        metadata: Owned(accts[5].clone()),
        treasury_mint: Owned(pubkeys::SOL.to_string()),
        escrow_payment_account: Owned(accts[6].clone()),
        seller_payment_receipt_account: Borrowed("magice_eden_haus"),
        buyer_receipt_token_account: Borrowed("magice_eden_haus"),
        authority: Owned(accts[8].clone()),
        auction_house: Owned(accts[9].clone()),
        auction_house_fee_account: Borrowed("magice_eden_haus"),
        auction_house_treasury: Borrowed("magice_eden_haus"),
        buyer_trade_state: Borrowed("magice_eden_haus"),
        seller_trade_state: Borrowed("magice_eden_haus"),
        free_trade_state: Borrowed("magice_eden_haus"),
        program_as_signer: Borrowed("magice_eden_haus"),
        escrow_payment_bump: 250,
        free_trade_state_bump: 250,
        program_as_signer_bump: 250,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    upsert_into_purchases_table(client, row.clone())
        .await
        .context("failed to insert listing!")?;

    client
        .db()
        .run(move |db| {
            insert_into(execute_sale_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert sell instruction ")?;
    Ok(())

}

pub(crate) async fn process_sale(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME instructions")?;

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = SellInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[3].clone()),
        metadata: Owned(accts[5].clone()),
        authority: Owned(accts[6].clone()),
        auction_house: Owned(accts[7].clone()),
        // TODO: make this optional
        auction_house_fee_account: Borrowed("magice_eden_haus"),
        // TODO: make this optional
        seller_trade_state: Borrowed("magice_eden_haus"),
        // TODO: make this optional
        free_seller_trader_state: Borrowed("magice_eden_haus"),
        // TODO: make this optional
        program_as_signer: Borrowed("magice_eden_haus"),
        trade_state_bump: params.trade_state_bump.try_into()?,
        free_trade_state_bump: 250,
        program_as_signer_bump: 250,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    upsert_into_listings_table(client, row.clone())
        .await
        .context("failed to insert listing!")?;

    client
        .db()
        .run(move |db| {
            insert_into(sell_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert sell instruction ")?;

    Ok(())
}

pub(crate) async fn process_buy(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME  instructions")?;

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
        treasury_mint: Owned(pubkeys::SOL.to_string()),
        token_account: Borrowed("magic_eden_haus"),// this isnt passed in by magicEden,
        metadata: Owned(accts[3].clone()),
        escrow_payment_account: Owned(accts[4].clone()),
        authority: Owned(accts[5].clone()),
        auction_house: Owned(accts[6].clone()),
        // TODO: make this optional?
        auction_house_fee_account: Borrowed("magic_eden_haus"),
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

    match discriminator {
        BUY => process_buy(client, &params, accounts, slot).await,
        SELL => process_sale(client, &params, accounts, slot).await,
        EXECUTE_SALE => process_execute_sale(client, &params, accounts, slot).await,
        _ => Ok(()),
    }
}
