use objects::candymachine::{
    CandyMachine, CandyMachineCollectionPda, CandyMachineConfigLine, CandyMachineCreator,
};
use scalars::PublicKey;
use tables::{candy_machine_collection_pdas, candy_machine_config_lines, candy_machine_creators};

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

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Option<CandyMachineCollectionPda>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Option<CandyMachineCollectionPda>> {
        let conn = self.db()?;

        let rows: Vec<models::CMCollectionPDA> = candy_machine_collection_pdas::table
            .filter(candy_machine_collection_pdas::candy_machine.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load candy machine collection pdas")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Vec<CandyMachineConfigLine>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Vec<CandyMachineConfigLine>> {
        let conn = self.db()?;

        let rows: Vec<models::CMConfigLine> = candy_machine_config_lines::table
            .filter(candy_machine_config_lines::address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to candy machine config lines")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.address.clone(), r.try_into()))
            .batch(addresses))
    }
}
