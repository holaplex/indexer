use borsh::BorshDeserialize;
use indexer_core::db::{
    insert_into, models::WithdrawFromTreasuryInstruction,
    tables::withdraw_from_treasury_instructions,
};
use mpl_auction_house::instruction::WithdrawFromTreasury;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = WithdrawFromTreasury::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 8 {
        debug!("invalid accounts for WithdrawFromTreasury instruction");
        return Ok(());
    }
    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = WithdrawFromTreasuryInstruction {
        treasury_mint: Owned(accts[0].clone()),
        authority: Owned(accts[1].clone()),
        treasury_withdrawal_destination: Owned(accts[2].clone()),
        auction_house_treasury: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        amount: params.amount.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(withdraw_from_treasury_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert withdraw from treasury instruction ")?;
    Ok(())
}
