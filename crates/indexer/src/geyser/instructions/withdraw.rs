use borsh::BorshDeserialize;
use indexer_core::db::{insert_into, models::WithdrawInstruction, tables::withdraw_instructions};
use mpl_auction_house::instruction::Withdraw;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = Withdraw::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 11 {
        debug!("invalid accounts for WithdrawInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = WithdrawInstruction {
        wallet: Owned(accts[0].clone()),
        receipt_account: Owned(accts[1].clone()),
        escrow_payment_account: Owned(accts[2].clone()),
        treasury_mint: Owned(accts[3].clone()),
        authority: Owned(accts[4].clone()),
        auction_house: Owned(accts[5].clone()),
        auction_house_fee_account: Owned(accts[6].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        amount: params.amount.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(withdraw_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert withdraw instruction ")?;
    Ok(())
}
