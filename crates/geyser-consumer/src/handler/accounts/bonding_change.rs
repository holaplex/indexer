use indexer_core::{
    db::{insert_into, models::BondingChange, tables::bonding_changes},
    prelude::*,
};
use spl_token_bonding::state::TokenBondingV0;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    slot: i64,
    bonding: TokenBondingV0,
) -> Result<()> {
    let row = BondingChange {
        address: Owned(key.to_string()),
        insert_ts: Local::now().naive_utc(),
        slot,
        current_reserves_from_bonding: i64::try_from(bonding.reserve_balance_from_bonding)
            .context("Reserves from bonding was too big to store")?,
        current_supply_from_bonding: i64::try_from(bonding.supply_from_bonding)
            .context("Supply from bonding was too big to store")?,
    };

    client
        .db()
        .run(move |db| insert_into(bonding_changes::table).values(&row).execute(db))
        .await
        .context("Failed to insert token bonding change")?;

    Ok(())
}
