use indexer_core::{
    bigdecimal,
    db::custom_types::{
        GovernanceAccountTypeEnum, InstructionExecutionFlagsEnum, MintMaxVoteEnum,
        OptionVoteResultEnum, ProposalStateEnum, ProposalVoteTypeEnum, VoteRecordV2VoteEnum,
        VoteThresholdEnum, VoteTippingEnum, VoteWeightV1Enum,
    },
};
use objects::wallet::Wallet;
use scalars::{
    markers::{GovernanceDelegate, GovernedAccount, TokenMint},
    PublicKey, I64, U64,
};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Governance {
    pub address: PublicKey<Governance>,
    pub account_type: GovernanceAccountType,
    pub realm: PublicKey<Realm>,
    pub governed_account: PublicKey<GovernedAccount>,
    pub proposals_count: I64,
    pub voting_proposal_count: i32,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPL Governance account")]
impl Governance {
    fn address(&self) -> &PublicKey<Governance> {
        &self.address
    }

    fn account_type(&self) -> &GovernanceAccountType {
        &self.account_type
    }

    fn governed_account(&self) -> &PublicKey<GovernedAccount> {
        &self.governed_account
    }

    pub fn proposals_count(&self) -> &I64 {
        &self.proposals_count
    }

    pub fn voting_proposal_count(&self) -> i32 {
        self.voting_proposal_count
    }

    pub async fn governance_config(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<GovernanceConfig>> {
        ctx.spl_governance_config_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn realm(&self, ctx: &AppContext) -> FieldResult<Option<Realm>> {
        ctx.spl_realm_loader
            .load(self.realm.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::Governance<'a>> for Governance {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Governance {
            address,
            account_type,
            realm,
            governed_account,
            proposals_count,
            voting_proposal_count,
            ..
        }: models::Governance,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            realm: realm.into_owned().into(),
            governed_account: governed_account.into_owned().into(),
            proposals_count: proposals_count.into(),
            voting_proposal_count: voting_proposal_count.try_into()?,
        })
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum GovernanceAccountType {
    Uninitialized,
    RealmV1,
    TokenOwnerRecordV1,
    GovernanceV1,
    ProgramGovernanceV1,
    ProposalV1,
    SignatoryRecordV1,
    VoteRecordV1,
    ProposalInstructionV1,
    MintGovernanceV1,
    TokenGovernanceV1,
    RealmConfig,
    VoteRecordV2,
    ProposalTransactionV2,
    ProposalV2,
    ProgramMetadata,
    RealmV2,
    TokenOwnerRecordV2,
    GovernanceV2,
    ProgramGovernanceV2,
    MintGovernanceV2,
    TokenGovernanceV2,
    SignatoryRecordV2,
}

impl From<GovernanceAccountTypeEnum> for GovernanceAccountType {
    fn from(v: GovernanceAccountTypeEnum) -> Self {
        match v {
            GovernanceAccountTypeEnum::Uninitialized => GovernanceAccountType::Uninitialized,
            GovernanceAccountTypeEnum::RealmV1 => GovernanceAccountType::RealmV1,
            GovernanceAccountTypeEnum::TokenOwnerRecordV1 => {
                GovernanceAccountType::TokenOwnerRecordV1
            },
            GovernanceAccountTypeEnum::GovernanceV1 => GovernanceAccountType::GovernanceV1,
            GovernanceAccountTypeEnum::ProgramGovernanceV1 => {
                GovernanceAccountType::ProgramGovernanceV1
            },
            GovernanceAccountTypeEnum::ProposalV1 => GovernanceAccountType::ProposalV1,
            GovernanceAccountTypeEnum::SignatoryRecordV1 => {
                GovernanceAccountType::SignatoryRecordV1
            },
            GovernanceAccountTypeEnum::VoteRecordV1 => GovernanceAccountType::VoteRecordV1,
            GovernanceAccountTypeEnum::ProposalInstructionV1 => {
                GovernanceAccountType::ProposalInstructionV1
            },
            GovernanceAccountTypeEnum::MintGovernanceV1 => GovernanceAccountType::MintGovernanceV1,
            GovernanceAccountTypeEnum::TokenGovernanceV1 => {
                GovernanceAccountType::TokenGovernanceV1
            },
            GovernanceAccountTypeEnum::RealmConfig => GovernanceAccountType::RealmConfig,
            GovernanceAccountTypeEnum::VoteRecordV2 => GovernanceAccountType::VoteRecordV2,
            GovernanceAccountTypeEnum::ProposalTransactionV2 => {
                GovernanceAccountType::ProposalTransactionV2
            },
            GovernanceAccountTypeEnum::ProposalV2 => GovernanceAccountType::ProposalV2,
            GovernanceAccountTypeEnum::ProgramMetadata => GovernanceAccountType::ProgramMetadata,
            GovernanceAccountTypeEnum::RealmV2 => GovernanceAccountType::RealmV2,
            GovernanceAccountTypeEnum::TokenOwnerRecordV2 => {
                GovernanceAccountType::TokenOwnerRecordV2
            },
            GovernanceAccountTypeEnum::GovernanceV2 => GovernanceAccountType::GovernanceV2,
            GovernanceAccountTypeEnum::ProgramGovernanceV2 => {
                GovernanceAccountType::ProgramGovernanceV2
            },
            GovernanceAccountTypeEnum::MintGovernanceV2 => GovernanceAccountType::MintGovernanceV2,
            GovernanceAccountTypeEnum::TokenGovernanceV2 => {
                GovernanceAccountType::TokenGovernanceV2
            },
            GovernanceAccountTypeEnum::SignatoryRecordV2 => {
                GovernanceAccountType::SignatoryRecordV2
            },
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct GovernanceConfig {
    pub governance_address: PublicKey<Governance>,
    pub vote_threshold_type: VoteThreshold,
    pub vote_threshold_percentage: i32,
    pub min_community_weight_to_create_proposal: U64,
    pub min_instruction_hold_up_time: I64,
    pub max_voting_time: I64,
    pub vote_tipping: VoteTipping,
    pub proposal_cool_off_time: I64,
    pub min_council_weight_to_create_proposal: I64,
}

impl<'a> TryFrom<models::GovernanceConfig<'a>> for GovernanceConfig {
    type Error = bigdecimal::ParseBigDecimalError;

    fn try_from(
        models::GovernanceConfig {
            governance_address,
            vote_threshold_type,
            vote_threshold_percentage,
            min_community_weight_to_create_proposal,
            min_instruction_hold_up_time,
            max_voting_time,
            vote_tipping,
            proposal_cool_off_time,
            min_council_weight_to_create_proposal,
            ..
        }: models::GovernanceConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            governance_address: governance_address.into_owned().into(),
            vote_threshold_type: vote_threshold_type.into(),
            vote_threshold_percentage: vote_threshold_percentage.into(),
            min_community_weight_to_create_proposal: min_community_weight_to_create_proposal
                .try_into()?,
            min_instruction_hold_up_time: min_instruction_hold_up_time.into(),
            max_voting_time: max_voting_time.into(),
            vote_tipping: vote_tipping.into(),
            proposal_cool_off_time: proposal_cool_off_time.into(),
            min_council_weight_to_create_proposal: min_council_weight_to_create_proposal.into(),
        })
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum VoteThreshold {
    YesVote,
    Quorum,
}

impl From<VoteThresholdEnum> for VoteThreshold {
    fn from(v: VoteThresholdEnum) -> Self {
        match v {
            VoteThresholdEnum::YesVote => VoteThreshold::YesVote,
            VoteThresholdEnum::Quorum => VoteThreshold::Quorum,
        }
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum VoteTipping {
    Strict,
    Early,
    Disabled,
}

impl From<VoteTippingEnum> for VoteTipping {
    fn from(v: VoteTippingEnum) -> Self {
        match v {
            VoteTippingEnum::Strict => VoteTipping::Strict,
            VoteTippingEnum::Early => VoteTipping::Early,
            VoteTippingEnum::Disabled => VoteTipping::Disabled,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Realm {
    pub address: PublicKey<Realm>,
    pub account_type: GovernanceAccountType,
    pub community_mint: PublicKey<TokenMint>,
    pub voting_proposal_count: i32,
    pub authority: Option<PublicKey<Wallet>>,
    pub name: String,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance Realm account")]
impl Realm {
    fn address(&self) -> &PublicKey<Realm> {
        &self.address
    }

    fn account_type(&self) -> &GovernanceAccountType {
        &self.account_type
    }

    fn community_mint(&self) -> &PublicKey<TokenMint> {
        &self.community_mint
    }

    pub fn voting_proposal_count(&self) -> i32 {
        self.voting_proposal_count
    }

    pub fn authority(&self) -> &Option<PublicKey<Wallet>> {
        &self.authority
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn realm_config(&self, ctx: &AppContext) -> FieldResult<Option<RealmConfig>> {
        ctx.spl_realm_config_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::Realm<'a>> for Realm {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Realm {
            address,
            account_type,
            community_mint,
            voting_proposal_count,
            authority,
            name,
            ..
        }: models::Realm,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            community_mint: community_mint.into_owned().into(),
            voting_proposal_count: voting_proposal_count.try_into()?,
            authority: authority.map(Into::into),
            name: name.to_string(),
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct RealmConfig {
    pub realm_address: PublicKey<Realm>,
    pub use_community_voter_weight_addin: bool,
    pub use_max_community_voter_weight_addin: bool,
    pub min_community_weight_to_create_governance: U64,
    pub community_mint_max_vote_weight_source: MintMaxVoteWeightSource,
    pub community_mint_max_vote_weight: I64,
    pub council_mint: Option<PublicKey<TokenMint>>,
}

impl<'a> TryFrom<models::RealmConfig<'a>> for RealmConfig {
    type Error = bigdecimal::ParseBigDecimalError;

    fn try_from(
        models::RealmConfig {
            realm_address,
            use_community_voter_weight_addin,
            use_max_community_voter_weight_addin,
            min_community_weight_to_create_governance,
            community_mint_max_vote_weight_source,
            community_mint_max_vote_weight,
            council_mint,
            ..
        }: models::RealmConfig,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            realm_address: realm_address.into_owned().into(),
            use_community_voter_weight_addin,
            use_max_community_voter_weight_addin,
            min_community_weight_to_create_governance: min_community_weight_to_create_governance
                .try_into()?,
            community_mint_max_vote_weight_source: community_mint_max_vote_weight_source.into(),
            community_mint_max_vote_weight: community_mint_max_vote_weight.into(),
            council_mint: council_mint.map(Into::into),
        })
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum MintMaxVoteWeightSource {
    SupplyFraction,
    Absolute,
}

impl From<MintMaxVoteEnum> for MintMaxVoteWeightSource {
    fn from(v: MintMaxVoteEnum) -> Self {
        match v {
            MintMaxVoteEnum::SupplyFraction => MintMaxVoteWeightSource::SupplyFraction,
            MintMaxVoteEnum::Absolute => MintMaxVoteWeightSource::Absolute,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoteRecordV1 {
    pub address: PublicKey<VoteRecordV1>,
    pub proposal: PublicKey<ProposalV1>,
    pub governing_token_owner: PublicKey<Wallet>,
    pub is_relinquished: bool,
    pub vote_type: VoteWeightV1,
    pub vote_weight: I64,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance VoteRecordV2 account")]
impl VoteRecordV1 {
    fn address(&self) -> &PublicKey<VoteRecordV1> {
        &self.address
    }

    fn governing_token_owner(&self) -> &PublicKey<Wallet> {
        &self.governing_token_owner
    }

    fn is_relinquished(&self) -> bool {
        self.is_relinquished
    }

    pub fn vote_weight(&self) -> &I64 {
        &self.vote_weight
    }

    pub fn vote_type(&self) -> &VoteWeightV1 {
        &self.vote_type
    }

    pub async fn proposal(&self, ctx: &AppContext) -> FieldResult<Option<ProposalV1>> {
        ctx.spl_proposalv1_loader
            .load(self.proposal.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn token_owner_records(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Vec<TokenOwnerRecord>> {
        ctx.spl_vote_record_token_owner_loader
            .load(self.governing_token_owner.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum VoteWeightV1 {
    Yes,
    No,
}

impl From<VoteWeightV1Enum> for VoteWeightV1 {
    fn from(v: VoteWeightV1Enum) -> Self {
        match v {
            VoteWeightV1Enum::Yes => VoteWeightV1::Yes,
            VoteWeightV1Enum::No => VoteWeightV1::No,
        }
    }
}

impl<'a> From<models::VoteRecordV1<'a>> for VoteRecordV1 {
    fn from(
        models::VoteRecordV1 {
            address,
            proposal,
            governing_token_owner,
            is_relinquished,
            vote_type,
            vote_weight,
            ..
        }: models::VoteRecordV1,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            proposal: proposal.into_owned().into(),
            governing_token_owner: governing_token_owner.into_owned().into(),
            is_relinquished,
            vote_type: vote_type.into(),
            vote_weight: vote_weight.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoteRecordV2 {
    pub address: PublicKey<VoteRecordV2>,
    pub proposal: PublicKey<ProposalV2>,
    pub governing_token_owner: PublicKey<Wallet>,
    pub is_relinquished: bool,
    pub voter_weight: I64,
    pub vote: Vote,
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum Vote {
    Approve,
    Deny,
    Abstain,
    Veto,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance VoteRecordV2 account")]
impl VoteRecordV2 {
    fn address(&self) -> &PublicKey<VoteRecordV2> {
        &self.address
    }

    fn governing_token_owner(&self) -> &PublicKey<Wallet> {
        &self.governing_token_owner
    }

    fn is_relinquished(&self) -> bool {
        self.is_relinquished
    }

    pub fn voter_weight(&self) -> &I64 {
        &self.voter_weight
    }

    pub fn vote(&self) -> &Vote {
        &self.vote
    }

    pub async fn approve_vote_choices(&self, ctx: &AppContext) -> FieldResult<Vec<VoteChoice>> {
        ctx.spl_approve_vote_choices_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn proposal(&self, ctx: &AppContext) -> FieldResult<Option<ProposalV2>> {
        ctx.spl_proposalv2_loader
            .load(self.proposal.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn token_owner_records(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Vec<TokenOwnerRecord>> {
        ctx.spl_vote_record_token_owner_loader
            .load(self.governing_token_owner.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<models::VoteRecordV2<'a>> for VoteRecordV2 {
    fn from(
        models::VoteRecordV2 {
            address,
            proposal,
            governing_token_owner,
            is_relinquished,
            voter_weight,
            vote,
            ..
        }: models::VoteRecordV2,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            proposal: proposal.into_owned().into(),
            governing_token_owner: governing_token_owner.into_owned().into(),
            is_relinquished,
            voter_weight: voter_weight.into(),
            vote: vote.into(),
        }
    }
}

impl From<VoteRecordV2VoteEnum> for Vote {
    fn from(v: VoteRecordV2VoteEnum) -> Self {
        match v {
            VoteRecordV2VoteEnum::Approve => Vote::Approve,
            VoteRecordV2VoteEnum::Deny => Vote::Deny,
            VoteRecordV2VoteEnum::Abstain => Vote::Abstain,
            VoteRecordV2VoteEnum::Veto => Vote::Veto,
        }
    }
}

#[derive(derive_more::From, juniper::GraphQLUnion)]
#[graphql(
  Context = AppContext,
)]
pub enum VoteRecord {
    VoteRecordV1(VoteRecordV1),
    VoteRecordV2(VoteRecordV2),
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid vote_record variant")]
pub struct TryFromError;

impl TryFrom<models::VoteRecord> for VoteRecord {
    type Error = TryFromError;

    fn try_from(
        models::VoteRecord {
            address,
            account_type,
            proposal,
            governing_token_owner,
            is_relinquished,
            vote_type,
            vote_weight,
            voter_weight,
            vote,
        }: models::VoteRecord,
    ) -> Result<Self, Self::Error> {
        match account_type {
            GovernanceAccountTypeEnum::VoteRecordV1 => Ok(Self::VoteRecordV1(VoteRecordV1 {
                address: address.into(),
                proposal: proposal.into(),
                governing_token_owner: governing_token_owner.into(),
                is_relinquished,
                vote_type: vote_type.unwrap().into(),
                vote_weight: vote_weight.unwrap().into(),
            })),
            GovernanceAccountTypeEnum::VoteRecordV2 => Ok(Self::VoteRecordV2(VoteRecordV2 {
                address: address.into(),
                proposal: proposal.into(),
                governing_token_owner: governing_token_owner.into(),
                is_relinquished,
                vote: vote.unwrap().into(),
                voter_weight: voter_weight.unwrap().into(),
            })),
            _ => Err(TryFromError),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct VoteChoice {
    pub address: PublicKey<VoteRecordV2>,
    pub rank: i32,
    pub weight_percentage: i32,
}

impl<'a> From<models::VoteChoice<'a>> for VoteChoice {
    fn from(
        models::VoteChoice {
            vote_record_v2_address,
            rank,
            weight_percentage,
            ..
        }: models::VoteChoice,
    ) -> Self {
        Self {
            address: vote_record_v2_address.into_owned().into(),
            rank: rank.into(),
            weight_percentage: weight_percentage.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenOwnerRecord {
    pub address: PublicKey<TokenOwnerRecord>,
    pub account_type: GovernanceAccountType,
    pub realm: PublicKey<Realm>,
    pub governing_token_mint: PublicKey<TokenMint>,
    pub governing_token_owner: PublicKey<Wallet>,
    pub governing_token_deposit_amount: I64,
    pub unrelinquished_votes_count: I64,
    pub total_votes_count: I64,
    pub outstanding_proposal_count: i32,
    pub governance_delegate: Option<PublicKey<GovernanceDelegate>>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance TokenOwnerRecord account")]
impl TokenOwnerRecord {
    fn address(&self) -> &PublicKey<TokenOwnerRecord> {
        &self.address
    }

    fn account_type(&self) -> &GovernanceAccountType {
        &self.account_type
    }

    fn governing_token_mint(&self) -> &PublicKey<TokenMint> {
        &self.governing_token_mint
    }

    fn governing_token_owner(&self) -> &PublicKey<Wallet> {
        &self.governing_token_owner
    }

    fn governing_token_deposit_amount(&self) -> &I64 {
        &self.governing_token_deposit_amount
    }

    fn unrelinquished_votes_count(&self) -> &I64 {
        &self.unrelinquished_votes_count
    }

    fn total_votes_count(&self) -> &I64 {
        &self.total_votes_count
    }

    fn outstanding_proposal_count(&self) -> i32 {
        self.outstanding_proposal_count
    }

    fn governance_delegate(&self) -> &Option<PublicKey<GovernanceDelegate>> {
        &self.governance_delegate
    }

    pub async fn realm(&self, ctx: &AppContext) -> FieldResult<Option<Realm>> {
        ctx.spl_realm_loader
            .load(self.realm.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<models::TokenOwnerRecord<'a>> for TokenOwnerRecord {
    fn from(
        models::TokenOwnerRecord {
            address,
            account_type,
            realm,
            governing_token_mint,
            governing_token_owner,
            governing_token_deposit_amount,
            unrelinquished_votes_count,
            total_votes_count,
            outstanding_proposal_count,
            governance_delegate,
            ..
        }: models::TokenOwnerRecord,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            realm: realm.into_owned().into(),
            governing_token_mint: governing_token_mint.into_owned().into(),
            governing_token_owner: governing_token_owner.into_owned().into(),
            governing_token_deposit_amount: governing_token_deposit_amount.into(),
            unrelinquished_votes_count: unrelinquished_votes_count.into(),
            total_votes_count: total_votes_count.into(),
            outstanding_proposal_count: outstanding_proposal_count.into(),
            governance_delegate: governance_delegate.map(Into::into),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignatoryRecord {
    pub address: PublicKey<SignatoryRecord>,
    pub account_type: GovernanceAccountType,
    pub proposal: PublicKey<Proposal>,
    pub signatory: PublicKey<Wallet>,
    pub signed_off: bool,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance SignatoryRecord account")]
impl SignatoryRecord {
    fn address(&self) -> &PublicKey<SignatoryRecord> {
        &self.address
    }

    fn account_type(&self) -> &GovernanceAccountType {
        &self.account_type
    }

    fn signatory(&self) -> &PublicKey<Wallet> {
        &self.signatory
    }

    fn signed_off(&self) -> bool {
        self.signed_off
    }

    pub async fn proposal(&self, ctx: &AppContext) -> FieldResult<Option<Proposal>> {
        ctx.spl_proposal_loader
            .load(self.proposal.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<models::SignatoryRecord<'a>> for SignatoryRecord {
    fn from(
        models::SignatoryRecord {
            address,
            account_type,
            proposal,
            signatory,
            signed_off,
            ..
        }: models::SignatoryRecord,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            proposal: proposal.into_owned().into(),
            signatory: signatory.into_owned().into(),
            signed_off,
        }
    }
}

#[derive(Debug, Clone, derive_more::From, juniper::GraphQLUnion)]
#[graphql(
  Context = AppContext,
)]
pub enum Proposal {
    ProposalV1(ProposalV1),
    ProposalV2(ProposalV2),
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid proposal variant")]
pub struct TryFromProposalError;

impl TryFrom<models::SplGovernanceProposal> for Proposal {
    type Error = TryFromProposalError;

    fn try_from(
        models::SplGovernanceProposal {
            address,
            account_type,
            governance,
            governing_token_mint,
            state,
            token_owner_record,
            signatories_count,
            signatories_signed_off_count,
            yes_votes_count,
            no_votes_count,
            instructions_executed_count,
            instructions_count,
            instructions_next_index,
            draft_at,
            signing_off_at,
            voting_at,
            voting_at_slot,
            voting_completed_at,
            executing_at,
            closed_at,
            execution_flags,
            max_vote_weight,
            max_voting_time,
            vote_threshold_type,
            vote_threshold_percentage,
            name,
            description_link,
            vote_type,
            deny_vote_weight,
            veto_vote_weight,
            abstain_vote_weight,
            start_voting_at,
        }: models::SplGovernanceProposal,
    ) -> Result<Self, Self::Error> {
        match account_type {
            GovernanceAccountTypeEnum::ProposalV1 => Ok(Self::ProposalV1(ProposalV1 {
                address: address.into(),
                account_type: account_type.into(),
                governance: governance.into(),
                governing_token_mint: governing_token_mint.into(),
                state: state.into(),
                token_owner_record: token_owner_record.into(),
                signatories_count: signatories_count.into(),
                signatories_signed_off_count: signatories_signed_off_count.into(),
                yes_votes_count: yes_votes_count.unwrap().into(),
                no_votes_count: no_votes_count.unwrap().into(),
                instructions_executed_count: instructions_executed_count.unwrap().into(),
                instructions_count: instructions_count.unwrap().into(),
                instructions_next_index: instructions_next_index.unwrap().into(),
                draft_at: DateTime::from_utc(draft_at, Utc),
                signing_off_at: signing_off_at.map(|v| DateTime::from_utc(v, Utc)),
                voting_at: voting_at.map(|v| DateTime::from_utc(v, Utc)),
                voting_at_slot: voting_at_slot.map(Into::into),
                voting_completed_at: voting_completed_at.map(|v| DateTime::from_utc(v, Utc)),
                executing_at: executing_at.map(|v| DateTime::from_utc(v, Utc)),
                closed_at: closed_at.map(|v| DateTime::from_utc(v, Utc)),
                execution_flags: execution_flags.into(),
                max_vote_weight: max_vote_weight.map(Into::into),
                vote_threshold_type: vote_threshold_type.map(Into::into),
                vote_threshold_percentage: vote_threshold_percentage.map(Into::into),
                name,
                description_link,
            })),
            GovernanceAccountTypeEnum::ProposalV2 => Ok(Self::ProposalV2(ProposalV2 {
                address: address.into(),
                account_type: account_type.into(),
                governance: governance.into(),
                governing_token_mint: governing_token_mint.into(),
                state: state.into(),
                token_owner_record: token_owner_record.into(),
                signatories_count: signatories_count.into(),
                signatories_signed_off_count: signatories_signed_off_count.into(),
                vote_type: vote_type.unwrap().into(),
                deny_vote_weight: deny_vote_weight.map(Into::into),
                veto_vote_weight: veto_vote_weight.map(Into::into),
                abstain_vote_weight: abstain_vote_weight.map(Into::into),
                start_voting_at: start_voting_at.map(|v| DateTime::from_utc(v, Utc)),
                draft_at: DateTime::from_utc(draft_at, Utc),
                signing_off_at: signing_off_at.map(|v| DateTime::from_utc(v, Utc)),
                voting_at: voting_at.map(|v| DateTime::from_utc(v, Utc)),
                voting_at_slot: voting_at_slot.map(Into::into),
                voting_completed_at: voting_completed_at.map(|v| DateTime::from_utc(v, Utc)),
                executing_at: executing_at.map(|v| DateTime::from_utc(v, Utc)),
                closed_at: closed_at.map(|v| DateTime::from_utc(v, Utc)),
                execution_flags: execution_flags.into(),
                max_vote_weight: max_vote_weight.map(Into::into),
                max_voting_time: max_voting_time.map(Into::into),
                vote_threshold_type: vote_threshold_type.map(Into::into),
                vote_threshold_percentage: vote_threshold_percentage.map(Into::into),
                name,
                description_link,
            })),
            _ => Err(TryFromProposalError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProposalV1 {
    pub address: PublicKey<ProposalV1>,
    pub account_type: GovernanceAccountType,
    pub governance: PublicKey<Governance>,
    pub governing_token_mint: PublicKey<TokenMint>,
    pub state: ProposalState,
    pub token_owner_record: PublicKey<TokenOwnerRecord>,
    pub signatories_count: i32,
    pub signatories_signed_off_count: i32,
    pub yes_votes_count: I64,
    pub no_votes_count: I64,
    pub instructions_executed_count: i32,
    pub instructions_count: i32,
    pub instructions_next_index: i32,
    pub draft_at: DateTime<Utc>,
    pub signing_off_at: Option<DateTime<Utc>>,
    pub voting_at: Option<DateTime<Utc>>,
    pub voting_at_slot: Option<I64>,
    pub voting_completed_at: Option<DateTime<Utc>>,
    pub executing_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub execution_flags: InstructionExecutionFlags,
    pub max_vote_weight: Option<I64>,
    pub vote_threshold_type: Option<VoteThreshold>,
    pub vote_threshold_percentage: Option<i32>,
    pub name: String,
    pub description_link: String,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance ProposalV1 account")]
impl ProposalV1 {
    fn address(&self) -> &PublicKey<ProposalV1> {
        &self.address
    }

    fn account_type(&self) -> &GovernanceAccountType {
        &self.account_type
    }

    fn governing_token_mint(&self) -> &PublicKey<TokenMint> {
        &self.governing_token_mint
    }

    fn state(&self) -> &ProposalState {
        &self.state
    }

    fn signatories_count(&self) -> i32 {
        self.signatories_count
    }

    fn signatories_signed_off_count(&self) -> i32 {
        self.signatories_signed_off_count
    }

    fn yes_votes_count(&self) -> &I64 {
        &self.yes_votes_count
    }

    fn no_votes_count(&self) -> &I64 {
        &self.no_votes_count
    }

    fn instructions_executed_count(&self) -> i32 {
        self.instructions_executed_count
    }
    fn instructions_count(&self) -> i32 {
        self.instructions_count
    }
    fn instructions_next_index(&self) -> i32 {
        self.instructions_next_index
    }

    fn draft_at(&self) -> DateTime<Utc> {
        self.draft_at
    }

    fn signing_off_at(&self) -> &Option<DateTime<Utc>> {
        &self.signing_off_at
    }

    fn voting_at(&self) -> &Option<DateTime<Utc>> {
        &self.voting_at
    }

    fn voting_at_slot(&self) -> &Option<I64> {
        &self.voting_at_slot
    }

    fn voting_completed_at(&self) -> &Option<DateTime<Utc>> {
        &self.voting_completed_at
    }

    fn executing_at(&self) -> &Option<DateTime<Utc>> {
        &self.executing_at
    }

    fn closed_at(&self) -> &Option<DateTime<Utc>> {
        &self.closed_at
    }

    fn execution_flags(&self) -> &InstructionExecutionFlags {
        &self.execution_flags
    }

    fn max_vote_weight(&self) -> &Option<I64> {
        &self.max_vote_weight
    }

    fn vote_threshold_type(&self) -> &Option<VoteThreshold> {
        &self.vote_threshold_type
    }

    fn vote_threshold_percentage(&self) -> &Option<i32> {
        &self.vote_threshold_percentage
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description_link
    }

    pub async fn governance(&self, ctx: &AppContext) -> FieldResult<Option<Governance>> {
        ctx.spl_governance_loader
            .load(self.governance.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn token_owner_record(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<TokenOwnerRecord>> {
        ctx.spl_token_owner_record_loader
            .load(self.token_owner_record.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<models::ProposalV1<'a>> for ProposalV1 {
    fn from(
        models::ProposalV1 {
            address,
            account_type,
            governance,
            governing_token_mint,
            state,
            token_owner_record,
            signatories_count,
            signatories_signed_off_count,
            yes_votes_count,
            no_votes_count,
            instructions_executed_count,
            instructions_count,
            instructions_next_index,
            draft_at,
            signing_off_at,
            voting_at,
            voting_at_slot,
            voting_completed_at,
            executing_at,
            closed_at,
            execution_flags,
            max_vote_weight,
            vote_threshold_type,
            vote_threshold_percentage,
            name,
            description_link,
            ..
        }: models::ProposalV1,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            governance: governance.into_owned().into(),
            governing_token_mint: governing_token_mint.into_owned().into(),
            state: state.into(),
            token_owner_record: token_owner_record.into_owned().into(),
            signatories_count: signatories_count.into(),
            signatories_signed_off_count: signatories_signed_off_count.into(),
            yes_votes_count: yes_votes_count.into(),
            no_votes_count: no_votes_count.into(),
            instructions_executed_count: instructions_executed_count.into(),
            instructions_count: instructions_count.into(),
            instructions_next_index: instructions_next_index.into(),
            draft_at: DateTime::from_utc(draft_at, Utc),
            signing_off_at: signing_off_at.map(|v| DateTime::from_utc(v, Utc)),
            voting_at: voting_at.map(|v| DateTime::from_utc(v, Utc)),
            voting_at_slot: voting_at_slot.map(Into::into),
            voting_completed_at: voting_completed_at.map(|v| DateTime::from_utc(v, Utc)),
            executing_at: executing_at.map(|v| DateTime::from_utc(v, Utc)),
            closed_at: closed_at.map(|v| DateTime::from_utc(v, Utc)),
            execution_flags: execution_flags.into(),
            max_vote_weight: max_vote_weight.map(Into::into),
            vote_threshold_type: vote_threshold_type.map(Into::into),
            vote_threshold_percentage: vote_threshold_percentage.map(Into::into),
            name: name.into_owned(),
            description_link: description_link.into_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProposalV2 {
    pub address: PublicKey<ProposalV2>,
    pub account_type: GovernanceAccountType,
    pub governance: PublicKey<Governance>,
    pub governing_token_mint: PublicKey<TokenMint>,
    pub state: ProposalState,
    pub token_owner_record: PublicKey<TokenOwnerRecord>,
    pub signatories_count: i32,
    pub signatories_signed_off_count: i32,
    pub vote_type: VoteType,
    pub deny_vote_weight: Option<I64>,
    pub veto_vote_weight: Option<I64>,
    pub abstain_vote_weight: Option<I64>,
    pub start_voting_at: Option<DateTime<Utc>>,
    pub draft_at: DateTime<Utc>,
    pub signing_off_at: Option<DateTime<Utc>>,
    pub voting_at: Option<DateTime<Utc>>,
    pub voting_at_slot: Option<I64>,
    pub voting_completed_at: Option<DateTime<Utc>>,
    pub executing_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub execution_flags: InstructionExecutionFlags,
    pub max_vote_weight: Option<I64>,
    pub max_voting_time: Option<I64>,
    pub vote_threshold_type: Option<VoteThreshold>,
    pub vote_threshold_percentage: Option<i32>,
    pub name: String,
    pub description_link: String,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance ProposalV2 account")]
impl ProposalV2 {
    fn address(&self) -> &PublicKey<ProposalV2> {
        &self.address
    }

    fn governing_token_mint(&self) -> &PublicKey<TokenMint> {
        &self.governing_token_mint
    }

    fn state(&self) -> &ProposalState {
        &self.state
    }

    fn signatories_count(&self) -> i32 {
        self.signatories_count
    }

    fn signatories_signed_off_count(&self) -> i32 {
        self.signatories_signed_off_count
    }

    fn vote_type(&self) -> &VoteType {
        &self.vote_type
    }

    fn deny_vote_weight(&self) -> &Option<I64> {
        &self.deny_vote_weight
    }

    fn veto_vote_weight(&self) -> &Option<I64> {
        &self.veto_vote_weight
    }

    fn abstain_vote_weight(&self) -> &Option<I64> {
        &self.abstain_vote_weight
    }

    fn start_voting_at(&self) -> &Option<DateTime<Utc>> {
        &self.start_voting_at
    }

    fn draft_at(&self) -> DateTime<Utc> {
        self.draft_at
    }

    fn signing_off_at(&self) -> &Option<DateTime<Utc>> {
        &self.signing_off_at
    }

    fn voting_at(&self) -> &Option<DateTime<Utc>> {
        &self.voting_at
    }

    fn voting_at_slot(&self) -> &Option<I64> {
        &self.voting_at_slot
    }

    fn voting_completed_at(&self) -> &Option<DateTime<Utc>> {
        &self.voting_completed_at
    }

    fn executing_at(&self) -> &Option<DateTime<Utc>> {
        &self.executing_at
    }

    fn closed_at(&self) -> &Option<DateTime<Utc>> {
        &self.closed_at
    }

    fn execution_flags(&self) -> &InstructionExecutionFlags {
        &self.execution_flags
    }

    fn max_vote_weight(&self) -> &Option<I64> {
        &self.max_vote_weight
    }

    fn max_voting_time(&self) -> &Option<I64> {
        &self.max_voting_time
    }

    fn vote_threshold_type(&self) -> &Option<VoteThreshold> {
        &self.vote_threshold_type
    }

    fn vote_threshold_percentage(&self) -> &Option<i32> {
        &self.vote_threshold_percentage
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description_link
    }

    pub async fn multi_choice(&self, ctx: &AppContext) -> FieldResult<Option<MultiChoice>> {
        ctx.spl_proposal_multi_choice_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn governance(&self, ctx: &AppContext) -> FieldResult<Option<Governance>> {
        ctx.spl_governance_loader
            .load(self.governance.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn token_owner_record(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<TokenOwnerRecord>> {
        ctx.spl_token_owner_record_loader
            .load(self.token_owner_record.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn proposal_options(&self, ctx: &AppContext) -> FieldResult<Vec<ProposalOption>> {
        ctx.spl_proposal_options_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<models::ProposalV2<'a>> for ProposalV2 {
    fn from(
        models::ProposalV2 {
            address,
            account_type,
            governance,
            governing_token_mint,
            state,
            token_owner_record,
            signatories_count,
            signatories_signed_off_count,
            vote_type,
            deny_vote_weight,
            veto_vote_weight,
            abstain_vote_weight,
            start_voting_at,
            draft_at,
            signing_off_at,
            voting_at,
            voting_at_slot,
            voting_completed_at,
            executing_at,
            closed_at,
            execution_flags,
            max_vote_weight,
            max_voting_time,
            vote_threshold_type,
            vote_threshold_percentage,
            name,
            description_link,
            ..
        }: models::ProposalV2,
    ) -> Self {
        Self {
            address: address.into_owned().into(),
            account_type: account_type.into(),
            governance: governance.into_owned().into(),
            governing_token_mint: governing_token_mint.into_owned().into(),
            state: state.into(),
            token_owner_record: token_owner_record.into_owned().into(),
            signatories_count: signatories_count.into(),
            signatories_signed_off_count: signatories_signed_off_count.into(),
            vote_type: vote_type.into(),
            deny_vote_weight: deny_vote_weight.map(Into::into),
            veto_vote_weight: veto_vote_weight.map(Into::into),
            abstain_vote_weight: abstain_vote_weight.map(Into::into),
            start_voting_at: start_voting_at.map(|v| DateTime::from_utc(v, Utc)),
            draft_at: DateTime::from_utc(draft_at, Utc),
            signing_off_at: signing_off_at.map(|v| DateTime::from_utc(v, Utc)),
            voting_at: voting_at.map(|v| DateTime::from_utc(v, Utc)),
            voting_at_slot: voting_at_slot.map(Into::into),
            voting_completed_at: voting_completed_at.map(|v| DateTime::from_utc(v, Utc)),
            executing_at: executing_at.map(|v| DateTime::from_utc(v, Utc)),
            closed_at: closed_at.map(|v| DateTime::from_utc(v, Utc)),
            execution_flags: execution_flags.into(),
            max_vote_weight: max_vote_weight.map(Into::into),
            max_voting_time: max_voting_time.map(Into::into),
            vote_threshold_type: vote_threshold_type.map(Into::into),
            vote_threshold_percentage: vote_threshold_percentage.map(Into::into),
            name: name.into_owned(),
            description_link: description_link.into_owned(),
        }
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum ProposalState {
    Draft,
    SigningOff,
    Voting,
    Succeeded,
    Executing,
    Completed,
    Cancelled,
    Defeated,
    ExecutingWithErrors,
}

impl From<ProposalStateEnum> for ProposalState {
    fn from(v: ProposalStateEnum) -> Self {
        match v {
            ProposalStateEnum::Draft => ProposalState::Draft,
            ProposalStateEnum::SigningOff => ProposalState::SigningOff,
            ProposalStateEnum::Voting => ProposalState::Voting,
            ProposalStateEnum::Succeeded => ProposalState::Succeeded,
            ProposalStateEnum::Executing => ProposalState::Executing,
            ProposalStateEnum::Completed => ProposalState::Completed,
            ProposalStateEnum::Cancelled => ProposalState::Cancelled,
            ProposalStateEnum::Defeated => ProposalState::Defeated,
            ProposalStateEnum::ExecutingWithErrors => ProposalState::ExecutingWithErrors,
        }
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum VoteType {
    SingleChoice,
    MultiChoice,
}

impl From<ProposalVoteTypeEnum> for VoteType {
    fn from(v: ProposalVoteTypeEnum) -> Self {
        match v {
            ProposalVoteTypeEnum::SingleChoice => VoteType::SingleChoice,
            ProposalVoteTypeEnum::MultiChoice => VoteType::MultiChoice,
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct MultiChoice {
    max_voter_options: i32,
    max_winning_options: i32,
}

impl<'a> From<models::MultiChoice<'a>> for MultiChoice {
    fn from(
        models::MultiChoice {
            max_voter_options,
            max_winning_options,
            ..
        }: models::MultiChoice,
    ) -> Self {
        Self {
            max_voter_options: max_voter_options.into(),
            max_winning_options: max_winning_options.into(),
        }
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum InstructionExecutionFlags {
    None,
    Ordered,
    UseTransaction,
}

impl From<InstructionExecutionFlagsEnum> for InstructionExecutionFlags {
    fn from(v: InstructionExecutionFlagsEnum) -> Self {
        match v {
            InstructionExecutionFlagsEnum::None => InstructionExecutionFlags::None,
            InstructionExecutionFlagsEnum::Ordered => InstructionExecutionFlags::Ordered,
            InstructionExecutionFlagsEnum::UseTransaction => {
                InstructionExecutionFlags::UseTransaction
            },
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct ProposalOption {
    pub proposal_address: PublicKey<ProposalV2>,
    pub label: String,
    pub vote_weight: I64,
    pub vote_result: OptionVoteResult,
    pub transactions_executed_count: i32,
    pub transactions_count: i32,
    pub transactions_next_index: i32,
}

impl<'a> From<models::ProposalOption<'a>> for ProposalOption {
    fn from(
        models::ProposalOption {
            proposal_address,
            label,
            vote_weight,
            vote_result,
            transactions_executed_count,
            transactions_count,
            transactions_next_index,
            ..
        }: models::ProposalOption,
    ) -> Self {
        Self {
            proposal_address: proposal_address.into_owned().into(),
            label: label.into_owned(),
            vote_weight: vote_weight.into(),
            vote_result: vote_result.into(),
            transactions_executed_count: transactions_executed_count.into(),
            transactions_count: transactions_count.into(),
            transactions_next_index: transactions_next_index.into(),
        }
    }
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum OptionVoteResult {
    None,
    Succeeded,
    Defeated,
}

impl From<OptionVoteResultEnum> for OptionVoteResult {
    fn from(v: OptionVoteResultEnum) -> Self {
        match v {
            OptionVoteResultEnum::None => OptionVoteResult::None,
            OptionVoteResultEnum::Succeeded => OptionVoteResult::Succeeded,
            OptionVoteResultEnum::Defeated => OptionVoteResult::Defeated,
        }
    }
}
