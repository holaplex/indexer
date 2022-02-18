use chrono::offset::Local;
use indexer_core::{
    db::{
        insert_into,
        models::{TokenAccount as TokenAccountModel, TokenTransfer},
        select,
        tables::{metadatas, token_accounts, token_transfers},
        update,
    },
    prelude::*,
    pubkeys::{kin, sol},
};
use spl_token::state::Account as TokenAccount;

use super::Client;
use crate::prelude::*;

pub async fn process_token_transfer(
    client: &Client,
    key: Pubkey,
    token_account: TokenAccount,
) -> Result<()> {
    let accts = client
        .db()
        .run(move |db| {
            token_accounts::table
                .filter(token_accounts::address.eq(&key.to_string()))
                .load::<TokenAccountModel>(db)
        })
        .await
        .context("Failed to check token_accounts!")?;
    let acct = accts
        .get(0)
        .context("Could not find existing token account!")?;
    if token_account.owner.to_string() != acct.owner_address {
        let row = TokenTransfer {
            owner_from: Owned(acct.owner_address.to_string()),
            owner_to: Owned(token_account.owner.to_string()),
            mint_address: Owned(token_account.mint.to_string()),
            transferred_at: Local::now().naive_utc(),
        };
        client
            .db()
            .run(move |db| {
                insert_into(token_transfers::table)
                    .values(&row)
                    .on_conflict_do_nothing()
                    .execute(db)
            })
            .await
            .context("failed to insert into token_transfers table")?;
        client
            .db()
            .run(move |db| {
                update(token_accounts::table)
                    .filter(token_accounts::address.eq(&key.to_string()))
                    .set(token_accounts::owner_address.eq(&token_account.owner.to_string()))
                    .execute(db)
            })
            .await
            .context("failed to update token account!")?;
    }

    Ok(())
}
pub async fn process(client: &Client, key: Pubkey, token_account: TokenAccount) -> Result<()> {
    if token_account.mint == sol() {
        return Ok(());
    }
    if token_account.mint == kin() {
        return Ok(());
    }

    let mint = token_account.mint.to_string();
    let is_present: bool = client
        .db()
        .run({
            let mint = mint.clone();
            |db| {
                select(exists(
                    metadatas::table.filter(metadatas::mint_address.eq(mint)),
                ))
                .get_result(db)
            }
        })
        .await
        .context("Failed to check mint address for existing mint")?;

    if !is_present {
        return Ok(());
    }

    let pubkey = key.to_string();

    let now = Local::now().naive_utc();
    let amount: i64 = token_account
        .amount
        .try_into()
        .context("Token amount was too big to store")?;
    let owner = token_account.owner.to_string();

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
    process_token_transfer(client, key, token_account).await
}
