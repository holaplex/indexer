use borsh::BorshDeserialize;
use indexer_core::db::{
    insert_into, models::WithdrawFromFeeInstruction, tables::withdraw_from_fee_instructions,
};
use mpl_auction_house::instruction::WithdrawFromFee;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = WithdrawFromFee::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 5 {
        debug!("invalid accounts for WithdrawFromFeeInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = WithdrawFromFeeInstruction {
        authority: Owned(accts[0].clone()),
        fee_withdrawal_destination: Owned(accts[1].clone()),
        auction_house_fee_account: Owned(accts[2].clone()),
        auction_house: Owned(accts[3].clone()),
        amount: params.amount.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(withdraw_from_fee_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert withdraw from fee instruction ")?;
    Ok(())
}
