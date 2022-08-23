use indexer_core::db::queries::spl_governance;
use objects::{
    spl_governance::{
        Governance, GovernanceConfig, MultiChoice, Proposal, ProposalOption, ProposalV1,
        ProposalV2, Realm, RealmConfig, TokenOwnerRecord, VoteChoice, VoteRecordV2,
    },
    wallet::Wallet,
};
use scalars::PublicKey;
use tables::{
    governance_configs, governances, proposal_options, proposal_vote_type_multi_choices,
    proposals_v1, proposals_v2, realm_configs, realms, token_owner_records,
    vote_record_v2_vote_approve_vote_choices,
};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<VoteRecordV2>, Vec<VoteChoice>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<VoteRecordV2>],
    ) -> TryBatchMap<PublicKey<VoteRecordV2>, Vec<VoteChoice>> {
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

#[async_trait]
impl TryBatchFn<PublicKey<ProposalV2>, Vec<ProposalOption>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<ProposalV2>],
    ) -> TryBatchMap<PublicKey<ProposalV2>, Vec<ProposalOption>> {
        let conn = self.db()?;

        let rows: Vec<models::ProposalOption> = proposal_options::table
            .filter(proposal_options::proposal_address.eq(any(addresses)))
            .select(proposal_options::all_columns)
            .load(&conn)
            .context("Failed to load proposal options")?;

        Ok(rows
            .into_iter()
            .map(|p| (p.proposal_address.clone(), p.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Realm>, Option<Realm>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Realm>],
    ) -> TryBatchMap<PublicKey<Realm>, Option<Realm>> {
        let conn = self.db()?;

        let rows: Vec<models::Realm> = realms::table
            .filter(realms::address.eq(any(addresses)))
            .select(realms::all_columns)
            .load(&conn)
            .context("Failed to load realms")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.address.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<ProposalV2>, Option<ProposalV2>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<ProposalV2>],
    ) -> TryBatchMap<PublicKey<ProposalV2>, Option<ProposalV2>> {
        let conn = self.db()?;

        let rows: Vec<models::ProposalV2> = proposals_v2::table
            .filter(proposals_v2::address.eq(any(addresses)))
            .select(proposals_v2::all_columns)
            .load(&conn)
            .context("Failed to load proposal")?;

        Ok(rows
            .into_iter()
            .map(|p| (p.address.clone(), p.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<ProposalV2>, Option<MultiChoice>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<ProposalV2>],
    ) -> TryBatchMap<PublicKey<ProposalV2>, Option<MultiChoice>> {
        let conn = self.db()?;

        let rows: Vec<models::MultiChoice> = proposal_vote_type_multi_choices::table
            .filter(proposal_vote_type_multi_choices::address.eq(any(addresses)))
            .select(proposal_vote_type_multi_choices::all_columns)
            .load(&conn)
            .context("Failed to load proposal multi choice vote type fields")?;

        Ok(rows
            .into_iter()
            .map(|p| (p.address.clone(), p.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Wallet>, Vec<TokenOwnerRecord>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Wallet>],
    ) -> TryBatchMap<PublicKey<Wallet>, Vec<TokenOwnerRecord>> {
        let conn = self.db()?;

        let rows: Vec<models::TokenOwnerRecord> = token_owner_records::table
            .filter(token_owner_records::governing_token_owner.eq(any(addresses)))
            .select(token_owner_records::all_columns)
            .load(&conn)
            .context("Failed to load token owner record")?;

        Ok(rows
            .into_iter()
            .map(|tor| (tor.governing_token_owner.clone(), tor.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Governance>, Option<Governance>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Governance>],
    ) -> TryBatchMap<PublicKey<Governance>, Option<Governance>> {
        let conn = self.db()?;

        let rows: Vec<models::Governance> = governances::table
            .filter(governances::address.eq(any(addresses)))
            .select(governances::all_columns)
            .load(&conn)
            .context("Failed to load spl governance")?;

        Ok(rows
            .into_iter()
            .map(|g| (g.address.clone(), g.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<TokenOwnerRecord>, Option<TokenOwnerRecord>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<TokenOwnerRecord>],
    ) -> TryBatchMap<PublicKey<TokenOwnerRecord>, Option<TokenOwnerRecord>> {
        let conn = self.db()?;

        let rows: Vec<models::TokenOwnerRecord> = token_owner_records::table
            .filter(token_owner_records::address.eq(any(addresses)))
            .select(token_owner_records::all_columns)
            .load(&conn)
            .context("Failed to load token owner record")?;

        Ok(rows
            .into_iter()
            .map(|tor| (tor.address.clone(), tor.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<ProposalV1>, Option<ProposalV1>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<ProposalV1>],
    ) -> TryBatchMap<PublicKey<ProposalV1>, Option<ProposalV1>> {
        let conn = self.db()?;

        let rows: Vec<models::ProposalV1> = proposals_v1::table
            .filter(proposals_v1::address.eq(any(addresses)))
            .select(proposals_v1::all_columns)
            .load(&conn)
            .context("Failed to load v1 proposal")?;

        Ok(rows
            .into_iter()
            .map(|p| (p.address.clone(), p.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Proposal>, Option<Proposal>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Proposal>],
    ) -> TryBatchMap<PublicKey<Proposal>, Option<Proposal>> {
        let conn = self.db()?;

        let proposals: Vec<models::SplGovernanceProposal> = spl_governance::proposals(
            &conn,
            addresses,
            Option::<Vec<PublicKey<Governance>>>::None,
        )?;

        Ok(proposals
            .into_iter()
            .map(|p| (p.address.clone(), p.try_into()))
            .batch(addresses))
    }
}
