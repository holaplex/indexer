use borsh::BorshDeserialize;
use indexer_core::db::{insert_into, models::BuyInstruction, tables::buy_instructions};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 14 {
        debug!("invalid accounts for BuyInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = BuyInstruction {
        wallet: Owned(accts[0].clone()),
        payment_account: Owned(accts[1].clone()),
        transfer_authority: Owned(accts[2].clone()),
        treasury_mint: Owned(accts[3].clone()),
        token_account: Owned(accts[4].clone()),
        metadata: Owned(accts[5].clone()),
        escrow_payment_account: Owned(accts[6].clone()),
        authority: Owned(accts[7].clone()),
        auction_house: Owned(accts[8].clone()),
        auction_house_fee_account: Owned(accts[9].clone()),
        buyer_trade_state: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    dbg!("{:?}", &row);

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
