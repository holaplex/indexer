//! Tribecca Simple-Voter program accounts indexing
use indexer_core::{
    db::{
        insert_into,
        models::{Electorate as ElectorateModel, TokenRecord as TokenRecordModel},
        tables::{electorates, token_records},
    },
    prelude::*,
};
use tribeca_simple_voter::{Electorate, TokenRecord};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_electorate(client: &Client, key: Pubkey, e: Electorate) -> Result<()> {
    let row = ElectorateModel {
        address: Owned(key.to_string()),
        bump: e.bump.try_into()?,
        base: Owned(e.base.to_string()),
        governor: Owned(e.governor.to_string()),
        gov_token_mint: Owned(e.gov_token_mint.to_string()),
        proposal_threshold: e.proposal_threshold.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(electorates::table)
                .values(&row)
                .on_conflict(electorates::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert electorate")?;

    Ok(())
}

pub(crate) async fn process_token_record(
    client: &Client,
    key: Pubkey,
    tr: TokenRecord,
) -> Result<()> {
    let row = TokenRecordModel {
        address: Owned(key.to_string()),
        bump: tr.bump.try_into()?,
        authority: Owned(tr.authority.to_string()),
        electorate: Owned(tr.electorate.to_string()),
        token_vault_key: Owned(tr.token_vault_key.to_string()),
        balance: tr.balance.try_into()?,
        unfinalized_votes: tr.unfinalized_votes.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(token_records::table)
                .values(&row)
                .on_conflict(token_records::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert token record")?;

    Ok(())
}
