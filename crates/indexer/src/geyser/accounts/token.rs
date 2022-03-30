use indexer_core::{
    db::{insert_into, models::TokenAccount as TokenAccountModel, tables::token_accounts, update},
    prelude::*,
};
use spl_token::state::Account as TokenAccount;

use super::Client;
use crate::prelude::*;

pub async fn process(
    client: &Client,
    key: Pubkey,
    token_account: TokenAccount,
    slot: u64,
) -> Result<()> {
    let pubkey = key.to_string();

    let amount: i64 = token_account
        .amount
        .try_into()
        .context("Token amount was too big to store")?;

    let owner = token_account.owner.to_string();
    let mint_address = token_account.mint.to_string();

    let values = TokenAccountModel {
        address: Owned(pubkey),
        amount,
        mint_address: Owned(mint_address),
        owner_address: Owned(owner),
        slot: Some(slot.try_into()?),
    };

    let incoming_slot: i64 = slot.try_into()?;

    client
        .db()
        .run(move |db| {
            let rows = token_accounts::table
                .select((
                    token_accounts::address,
                    token_accounts::mint_address,
                    token_accounts::owner_address,
                    token_accounts::amount,
                    token_accounts::slot,
                ))
                .filter(token_accounts::address.eq(key.to_string()))
                .load::<TokenAccountModel>(db)
                .context("failed to load token accounts!")?;

            match rows.get(0).and_then(|r| r.slot) {
                Some(indexed_slot) if incoming_slot > indexed_slot => {
                    db.build_transaction().read_write().run(|| {
                        update(
                            token_accounts::table
                                .filter(token_accounts::address.eq(values.clone().address)),
                        )
                        .set(&values)
                        .execute(db)
                        .context("transaction failed! unable to update token account when incoming slot > indexed slot")
                        .map(|_| ())
                    })
                },
                Some(_) => Ok(()),
                None => {
                    if amount == 1 {
                        db.build_transaction()
                            .read_write()
                            .run(|| {
                                update(token_accounts::table.filter(
                                    token_accounts::mint_address.eq(token_account.mint.to_string()),
                                ))
                                .set(token_accounts::amount.eq(0))
                                .execute(db)
                                .map(|_| ())
                            })
                            .context("transaction failed! unable to zero out token accounts amount")?;
                    };

                    db.build_transaction()
                        .read_write()
                        .run(|| {
                            insert_into(token_accounts::table)
                                .values(&values)
                                .on_conflict(token_accounts::address)
                                .do_update()
                                .set(&values)
                                .execute(db)
                                .map(|_| ())
                        })
                        .context("transaction failed! unable to insert token account")?;

                    Ok(())
                },
            }
        })
        .await
        .context("failed to insert token account!")?;
    Ok(())
}
