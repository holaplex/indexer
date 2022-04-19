use std::time::SystemTime;

use indexer_core::{
    db::{insert_into, models::BondingChange, tables::bonding_changes},
    prelude::*,
};
use spl_token_bonding::state::TokenBondingV0;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_token_bonding(
    client: &Client,
    key: Pubkey,
    slot: i64,
    bonding: TokenBondingV0,
) -> Result<()> {
    let row = BondingChange {
        address: Owned(key.to_string()),
        insert_ts: NaiveDateTime::from_timestamp(
            i64::try_from(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs(),
            )?,
            0,
        ),
        slot,
        current_reserves_from_bonding: i64::try_from(bonding.reserve_balance_from_bonding)
            .context("casting reserves")?,
        current_supply_from_bonding: i64::try_from(bonding.supply_from_bonding)
            .context("casting supply")?,
    };

    client
        .db()
        .run(move |db| insert_into(bonding_changes::table).values(&row).execute(db))
        .await
        .context("failed to insert token bonding")?;

    Ok(())
}
