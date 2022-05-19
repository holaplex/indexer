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

    let row = ExecuteSaleInstruction {
        buyer: Owned(
            accounts
                .get(0)
                .context("failed to get buyer pubkey")?
                .to_string(),
        ),
        seller: Owned(
            accounts
                .get(1)
                .context("failed to get seller pubkey")?
                .to_string(),
        ),
        token_account: Owned(
            accounts
                .get(2)
                .context("failed to get token account pubkey")?
                .to_string(),
        ),
        token_mint: Owned(
            accounts
                .get(3)
                .context("failed to get token mint pubkey")?
                .to_string(),
        ),
        metadata: Owned(
            accounts
                .get(4)
                .context("failed to get metadata pubkey")?
                .to_string(),
        ),
        treasury_mint: Owned(
            accounts
                .get(5)
                .context("failed to get treasury mint pubkey")?
                .to_string(),
        ),
        escrow_payment_account: Owned(
            accounts
                .get(6)
                .context("failed to get escrow payment account pubkey")?
                .to_string(),
        ),
        seller_payment_receipt_account: Owned(
            accounts
                .get(7)
                .context("failed to get seller payment receipt account pubkey")?
                .to_string(),
        ),
        buyer_receipt_token_account: Owned(
            accounts
                .get(8)
                .context("failed to get buyer receipt token account pubkey")?
                .to_string(),
        ),
        authority: Owned(
            accounts
                .get(9)
                .context("failed to get authority pubkey")?
                .to_string(),
        ),
        auction_house: Owned(
            accounts
                .get(10)
                .context("failed to get auction house pubkey")?
                .to_string(),
        ),
        auction_house_fee_account: Owned(
            accounts
                .get(11)
                .context("failed to get auction house fee account pubkey")?
                .to_string(),
        ),
        auction_house_treasury: Owned(
            accounts
                .get(12)
                .context("failed to get auction house treasury pubkey")?
                .to_string(),
        ),
        buyer_trade_state: Owned(
            accounts
                .get(13)
                .context("failed to get buyer trade state pubkey")?
                .to_string(),
        ),
        seller_trade_state: Owned(
            accounts
                .get(14)
                .context("failed to get seller trade state pubkey")?
                .to_string(),
        ),
        free_trade_state: Owned(
            accounts
                .get(15)
                .context("failed to get free trade state pubkey")?
                .to_string(),
        ),
        program_as_signer: Owned(
            accounts
                .get(19)
                .context("failed to get program as signer pubkey")?
                .to_string(),
        ),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    dbg!("{:?}", &row);

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
