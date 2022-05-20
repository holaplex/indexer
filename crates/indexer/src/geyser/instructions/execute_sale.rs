use borsh::BorshDeserialize;
use indexer_core::db::{
    insert_into, models::ExecuteSaleInstruction, tables::execute_sale_instructions,
};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    escrow_payment_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 23 {
        debug!("invalid accounts for ExecuteSaleInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = ExecuteSaleInstruction {
        buyer: Owned(accts[0].clone()),
        seller: Owned(accts[1].clone()),
        token_account: Owned(accts[2].clone()),
        token_mint: Owned(accts[3].clone()),
        metadata: Owned(accts[4].clone()),
        treasury_mint: Owned(accts[5].clone()),
        escrow_payment_account: Owned(accts[6].clone()),
        seller_payment_receipt_account: Owned(accts[7].clone()),
        buyer_receipt_token_account: Owned(accts[8].clone()),
        authority: Owned(accts[9].clone()),
        auction_house: Owned(accts[10].clone()),
        auction_house_fee_account: Owned(accts[11].clone()),
        auction_house_treasury: Owned(accts[12].clone()),
        buyer_trade_state: Owned(accts[13].clone()),
        seller_trade_state: Owned(accts[14].clone()),
        free_trade_state: Owned(accts[15].clone()),
        program_as_signer: Owned(accts[19].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(execute_sale_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert execute sale instruction ")?;
    Ok(())
}
