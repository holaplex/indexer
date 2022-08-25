use objects::candy_machine::{
    CandyMachine, CandyMachineCollectionPda, CandyMachineConfigLine, CandyMachineCreator,
    CandyMachineEndSetting, CandyMachineGateKeeperConfig, CandyMachineHiddenSetting,
    CandyMachineWhitelistMintSetting,
};
use scalars::PublicKey;
use tables::{
    candy_machine_collection_pdas, candy_machine_config_lines, candy_machine_creators,
    candy_machine_end_settings, candy_machine_gate_keeper_configs, candy_machine_hidden_settings,
    candy_machine_whitelist_mint_settings,
};

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
            .filter(candy_machine_config_lines::candy_machine_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to candy machine config lines")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Option<CandyMachineEndSetting>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Option<CandyMachineEndSetting>> {
        let conn = self.db()?;
        let rows: Vec<models::CMEndSetting> = candy_machine_end_settings::table
            .filter(candy_machine_end_settings::candy_machine_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load candy machine end settings")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Option<CandyMachineWhitelistMintSetting>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Option<CandyMachineWhitelistMintSetting>> {
        let conn = self.db()?;
        let rows: Vec<models::CMWhitelistMintSetting> =
            candy_machine_whitelist_mint_settings::table
                .filter(
                    candy_machine_whitelist_mint_settings::candy_machine_address.eq(any(addresses)),
                )
                .load(&conn)
                .context("Failed to load candy machine end settings")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Option<CandyMachineHiddenSetting>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Option<CandyMachineHiddenSetting>> {
        let conn = self.db()?;
        let rows: Vec<models::CMHiddenSetting> = candy_machine_hidden_settings::table
            .filter(candy_machine_hidden_settings::candy_machine_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load candy machine end settings")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CandyMachine>, Option<CandyMachineGateKeeperConfig>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CandyMachine>],
    ) -> TryBatchMap<PublicKey<CandyMachine>, Option<CandyMachineGateKeeperConfig>> {
        let conn = self.db()?;
        let rows: Vec<models::CMGateKeeperConfig> = candy_machine_gate_keeper_configs::table
            .filter(candy_machine_gate_keeper_configs::candy_machine_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load candy machine end settings")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.candy_machine_address.clone(), r.try_into()))
            .batch(addresses))
    }
}
