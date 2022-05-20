use borsh::BorshDeserialize;
use indexer_core::db::{insert_into, models::SellInstruction, tables::sell_instructions};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 12 {
        debug!("invalid accounts for SellInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = SellInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[1].clone()),
        metadata: Owned(accts[2].clone()),
        authority: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        auction_house_fee_account: Owned(accts[5].clone()),
        seller_trade_state: Owned(accts[6].clone()),
        free_seller_trader_state: Owned(accts[7].clone()),
        program_as_signer: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
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
            insert_into(sell_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert sell instruction ")?;
    Ok(())
}
