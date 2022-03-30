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

    if amount > 1 {
        return Ok(());
    }

    let rows = client
        .db()
        .run(move |db| {
            token_accounts::table
                .select((
                    token_accounts::address,
                    token_accounts::mint_address,
                    token_accounts::owner_address,
                    token_accounts::amount,
                    token_accounts::slot,
                ))
                .filter(token_accounts::address.eq(key.to_string()))
                .load::<TokenAccountModel>(db)
        })
        .await
        .context("failed to load token accounts!")?;

    let values = TokenAccountModel {
        address: Owned(pubkey),
        amount,
        mint_address: Owned(token_account.mint.to_string()),
        owner_address: Owned(owner),
        slot: Some(slot.try_into()?),
    };

    let incoming_slot: i64 = slot.try_into()?;

    match rows.get(0).and_then(|r| r.slot) {
        Some(indexed_slot) if incoming_slot > indexed_slot => {
            client
                .db()
                .run(move |db| {
                    update(
                        token_accounts::table
                            .filter(token_accounts::address.eq(values.clone().address)),
                    )
                    .set(&values)
                    .execute(db)
                })
                .await
                .context("failed to update token account")?;
        },
        Some(_) => (),
        None => {
            client
                .db()
                .run(move |db| {
                    insert_into(token_accounts::table)
                        .values(&values)
                        .on_conflict(token_accounts::address)
                        .do_update()
                        .set(&values)
                        .execute(db)
                })
                .await
                .context("failed to insert token account")?;
        },
    }

    Ok(())
}
