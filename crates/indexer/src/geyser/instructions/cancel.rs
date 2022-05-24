use borsh::BorshDeserialize;
use indexer_core::db::{
    insert_into,
    models::CancelInstruction,
    select,
    tables::{cancel_instructions, current_metadata_owners, listings, offers},
    update,
};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    buyer_price: u64,
    token_size: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 8 {
        debug!("invalid accounts for DepositInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = CancelInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[1].clone()),
        token_mint: Owned(accts[2].clone()),
        authority: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        auction_house_fee_account: Owned(accts[5].clone()),
        trade_state: Owned(accts[6].clone()),
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(cancel_instructions::table)
                .values(&row)
                .execute(db)?;
            db.build_transaction().read_write().run(|| {
                let wallet_is_owner = select(exists(
                    current_metadata_owners::table.filter(
                        current_metadata_owners::owner_address
                            .eq(row.wallet.clone())
                            .and(
                                current_metadata_owners::token_account_address
                                    .eq(row.token_account.clone()),
                            ),
                    ),
                ))
                .get_result::<bool>(db);

                if Ok(true) == wallet_is_owner {
                    update(
                        listings::table.filter(
                            listings::trade_state
                                .eq(row.trade_state.clone())
                                .and(listings::auction_house.eq(row.auction_house.clone()))
                                .and(listings::bookkeeper.eq(row.wallet.clone()))
                                .and(listings::price.eq(row.buyer_price))
                                .and(listings::token_size.eq(row.token_size)),
                        ),
                    )
                    .set(listings::canceled_at.eq(Some(row.created_at)))
                    .execute(db)
                } else {
                    update(
                        offers::table.filter(
                            offers::trade_state
                                .eq(row.trade_state.clone())
                                .and(offers::auction_house.eq(row.auction_house.clone()))
                                .and(offers::token_account.eq(row.token_account.clone()))
                                .and(offers::price.eq(row.buyer_price))
                                .and(offers::token_size.eq(row.token_size)),
                        ),
                    )
                    .set(offers::canceled_at.eq(Some(row.created_at)))
                    .execute(db)
                }
            })
        })
        .await
        .context("failed to insert cancel instruction ")?;

    Ok(())
}
