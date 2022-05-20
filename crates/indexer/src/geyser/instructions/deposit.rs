use borsh::BorshDeserialize;
use indexer_core::db::{insert_into, models::DepositInstruction, tables::deposit_instructions};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    escrow_payment_bump: u8,
    amount: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 11 {
        debug!("invalid accounts for DepositInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = DepositInstruction {
        wallet: Owned(accts[0].clone()),
        payment_account: Owned(accts[1].clone()),
        transfer_authority: Owned(accts[2].clone()),
        escrow_payment_account: Owned(accts[3].clone()),
        treasury_mint: Owned(accts[4].clone()),
        authority: Owned(accts[5].clone()),
        auction_house: Owned(accts[6].clone()),
        auction_house_fee_account: Owned(accts[7].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        amount: params.amount.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    dbg!("{:?}", &row);

    client
        .db()
        .run(move |db| {
            insert_into(deposit_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert deposit instruction ")?;
    Ok(())
}
