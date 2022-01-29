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

pub async fn process(client: &Client, pubkey: Pubkey, token_account_data: Vec<u8>) -> Result<()> {
    info!("token account {:?}", token_account_data);

    // let mint = token_account.mint.to_string();

    // let is_present: bool = client
    //     .db({
    //         let mint = mint.clone();
    //         |db| {
    //             select(exists(
    //                 metadatas::table.filter(metadatas::mint_address.eq(mint)),
    //             ))
    //             .get_result(db)
    //         }
    //     })
    //     .await
    //     .context("Failed to check mint address for existing mint")?;

    // if !is_present {
    //     return Ok(());
    // }

    // let pubkey = pubkey.to_string();
    // let amount: i64 = token_account
    //     .amount
    //     .try_into()
    //     .context("Token amount was too big to store")?;
    // let owner = token_account.owner.to_string();
    // let now = Local::now().naive_utc();

    // let values = TokenAccountModel {
    //     address: Owned(pubkey),
    //     amount,
    //     mint_address: Owned(mint),
    //     owner_address: Owned(owner),
    //     updated_at: now,
    // };

    // client
    //     .db(move |db| {
    //         insert_into(token_accounts::table)
    //             .values(&values)
    //             .on_conflict(token_accounts::address)
    //             .do_update()
    //             .set(&values)
    //             .execute(db)
    //     })
    //     .await
    //     .context("failed to insert token account")?;

    Ok(())
}
