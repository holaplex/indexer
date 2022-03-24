use chrono::offset::Local;
use indexer_core::{
    db::{insert_into, models::TokenAccount as TokenAccountModel, tables::token_accounts},
    prelude::*,
};
use spl_token::state::Account as TokenAccount;

use super::Client;
use crate::prelude::*;

pub async fn process(client: &Client, key: Pubkey, token_account: TokenAccount) -> Result<()> {
    let pubkey = key.to_string();

    let now = Local::now().naive_utc();
    let amount: i64 = token_account
        .amount
        .try_into()
        .context("Token amount was too big to store")?;
    let owner = token_account.owner.to_string();

    if amount > 1 {
        return Ok(());
    }

    let values = TokenAccountModel {
        address: Owned(pubkey),
        amount,
        mint_address: Owned(token_account.mint.to_string()),
        owner_address: Owned(owner),
        updated_at: now,
    };

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

    Ok(())
}
