use objects::candymachine::{CandyMachine, CandyMachineCreator};
use scalars::PublicKey;
use tables::candy_machine_creators;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Vec<CandyMachineCreator>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Vec<CandyMachineCreator>> {
        let conn = self.db()?;

        let rows: Vec<models::CMCreator> = candy_machine_creators::table
            .filter(candy_machine_creators::candy_machine_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to candy machine creators")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}
