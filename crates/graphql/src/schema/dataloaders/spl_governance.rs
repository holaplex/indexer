use objects::spl_governance::{
    Governance, GovernanceConfig, Realm, RealmConfig, VoteChoice, VoteRecord,
};
use scalars::PublicKey;
use tables::{governance_configs, realm_configs, vote_record_v2_vote_approve_vote_choices};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<VoteRecord>, Vec<VoteChoice>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<VoteRecord>],
    ) -> TryBatchMap<PublicKey<VoteRecord>, Vec<VoteChoice>> {
        let conn = self.db()?;

        let rows: Vec<models::VoteChoice> = vote_record_v2_vote_approve_vote_choices::table
            .filter(
                vote_record_v2_vote_approve_vote_choices::vote_record_v2_address.eq(any(addresses)),
            )
            .select(vote_record_v2_vote_approve_vote_choices::all_columns)
            .load(&conn)
            .context("Failed to load Approve Vote Choices")?;

        Ok(rows
            .into_iter()
            .map(|vc| (vc.vote_record_v2_address.clone(), vc.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Governance>, Option<GovernanceConfig>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Governance>],
    ) -> TryBatchMap<PublicKey<Governance>, Option<GovernanceConfig>> {
        let conn = self.db()?;

        let rows: Vec<models::GovernanceConfig> = governance_configs::table
            .filter(governance_configs::governance_address.eq(any(addresses)))
            .select(governance_configs::all_columns)
            .load(&conn)
            .context("Failed to load governance config")?;

        Ok(rows
            .into_iter()
            .map(|gc| (gc.governance_address.clone(), gc.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Realm>, Option<RealmConfig>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Realm>],
    ) -> TryBatchMap<PublicKey<Realm>, Option<RealmConfig>> {
        let conn = self.db()?;

        let rows: Vec<models::RealmConfig> = realm_configs::table
            .filter(realm_configs::realm_address.eq(any(addresses)))
            .select(realm_configs::all_columns)
            .load(&conn)
            .context("Failed to load realm config")?;

        Ok(rows
            .into_iter()
            .map(|rc| (rc.realm_address.clone(), rc.try_into()))
            .batch(addresses))
    }
}
