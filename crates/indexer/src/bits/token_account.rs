use chrono::offset::Local;
use indexer_core::{
    db::{
        insert_into,
        models::TokenAccount as TokenAccountModel,
        select,
        tables::{metadatas, token_accounts},
    },
    prelude::*,
};
use spl_token::state::Account as TokenAccount;

use crate::{prelude::*, Client};

pub fn process(client: &Client, pubkey: Pubkey, token_account: TokenAccount) -> Result<()> {
    let mint = token_account.mint.to_string();
    let owner = token_account.owner.to_string();
    let pubkey = pubkey.to_string();
    let db = client.db()?;
    let now = Local::now().naive_utc();

    if !select(exists(
        metadatas::table.filter(metadatas::mint_address.eq(&mint)),
    ))
    .get_result(&db)
    .context("Failed to check mint address for existing mint")?
    {
        return Ok(());
    }

    let amount = token_account.amount as i64;

    let values = TokenAccountModel {
        address: Borrowed(&pubkey),
        amount,
        mint_address: Borrowed(&mint),
        owner_address: Borrowed(&owner),
        updated_at: now,
    };

    insert_into(token_accounts::table)
        .values(&values)
        .on_conflict(token_accounts::address)
        .do_update()
        .set(&values)
        .execute(&db)
        .context("failed to insert token account")?;

    Ok(())
}
