use indexer_core::db::custom_types::{
    InstructionExecutionFlagsEnum, MintMaxVoteEnum, OptionVoteResultEnum, ProposalStateEnum,
    ProposalVoteTypeEnum, VoteRecordV2VoteEnum, VoteThresholdEnum, VoteTippingEnum,
};
use scalars::{
    markers::{
        CommunityMint, CouncilMint, GovernanceDelegate, GovernedAccount, GoverningTokenOwner,
        TokenMint,
    },
    PublicKey, I64,
};

use super::prelude::*;
use crate::schema::objects::wallet::Wallet;

#[derive(Debug, Clone)]
pub struct Governance {
    pub address: PublicKey<Governance>,
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

    fn realm(&self) -> &PublicKey<Realm> {
        &self.realm
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
        ctx.governance_config_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::Governance<'a>> for Governance {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Governance {
            address,
            realm,
            governed_account,
            proposals_count,
            voting_proposal_count,
            ..
        }: models::Governance,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            realm: realm.into_owned().into(),
            governed_account: governed_account.into_owned().into(),
            proposals_count: proposals_count.into(),
            voting_proposal_count: voting_proposal_count.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    pub governance_address: PublicKey<Governance>,
    pub vote_threshold_type: VoteThreshold,
    pub vote_threshold_percentage: i32,
    pub min_community_weight_to_create_proposal: I64,
    pub min_instruction_hold_up_time: I64,
    pub max_voting_time: I64,
    pub vote_tipping: VoteTipping,
    pub proposal_cool_off_time: I64,
    pub min_council_weight_to_create_proposal: I64,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPL Governance Config")]
impl GovernanceConfig {
    fn governance_address(&self) -> &PublicKey<Governance> {
        &self.governance_address
    }

    fn vote_threshold_type(&self) -> &VoteThreshold {
        &self.vote_threshold_type
    }

    fn vote_threshold_percentage(&self) -> i32 {
        self.vote_threshold_percentage
    }

    pub fn min_community_weight_to_create_proposal(&self) -> &I64 {
        &self.min_community_weight_to_create_proposal
    }

    pub fn min_instruction_hold_up_time(&self) -> &I64 {
        &self.min_instruction_hold_up_time
    }

    pub fn max_voting_time(&self) -> &I64 {
        &self.max_voting_time
    }

    pub fn vote_tipping(&self) -> &VoteTipping {
        &self.vote_tipping
    }

    pub fn proposal_cool_off_time(&self) -> &I64 {
        &self.proposal_cool_off_time
    }

    pub fn min_council_weight_to_create_proposal(&self) -> &I64 {
        &self.min_council_weight_to_create_proposal
    }
}

impl<'a> TryFrom<models::GovernanceConfig<'a>> for GovernanceConfig {
    type Error = std::num::TryFromIntError;
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
            min_community_weight_to_create_proposal: min_community_weight_to_create_proposal.into(),
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
    pub community_mint: PublicKey<CommunityMint>,
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

    fn community_mint(&self) -> &PublicKey<CommunityMint> {
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
        ctx.realm_config_loader
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
            community_mint,
            voting_proposal_count,
            authority,
            name,
            ..
        }: models::Realm,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            community_mint: community_mint.into_owned().into(),
            voting_proposal_count: voting_proposal_count.try_into()?,
            authority: authority.map(Into::into),
            name: name.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RealmConfig {
    pub realm_address: PublicKey<Realm>,
    pub use_community_voter_weight_addin: bool,
    pub use_max_community_voter_weight_addin: bool,
    pub min_community_weight_to_create_governance: I64,
    pub community_mint_max_vote_weight_source: MintMaxVoteWeightSource,
    pub community_mint_max_vote_weight: I64,
    pub council_mint: Option<PublicKey<CouncilMint>>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance Realm Config")]
impl RealmConfig {
    fn realm_address(&self) -> &PublicKey<Realm> {
        &self.realm_address
    }

    fn use_community_voter_weight_addin(&self) -> bool {
        self.use_community_voter_weight_addin
    }

    fn use_max_community_voter_weight_addin(&self) -> bool {
        self.use_max_community_voter_weight_addin
    }

    pub fn min_community_weight_to_create_governance(&self) -> &I64 {
        &self.min_community_weight_to_create_governance
    }

    pub fn community_mint_max_vote_weight_source(&self) -> &MintMaxVoteWeightSource {
        &self.community_mint_max_vote_weight_source
    }

    pub fn community_mint_max_vote_weight(&self) -> &I64 {
        &self.community_mint_max_vote_weight
    }

    pub fn council_mint(&self) -> &Option<PublicKey<CouncilMint>> {
        &self.council_mint
    }
}

impl<'a> TryFrom<models::RealmConfig<'a>> for RealmConfig {
    type Error = std::num::TryFromIntError;
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
                .into(),
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
pub struct VoteRecord {
    pub address: PublicKey<VoteRecord>,
    pub proposal: PublicKey<Proposal>,
    pub governing_token_owner: PublicKey<GoverningTokenOwner>,
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
#[graphql(description = "SPLGovernance VoteRecord account")]
impl VoteRecord {
    fn address(&self) -> &PublicKey<VoteRecord> {
        &self.address
    }

    fn proposal(&self) -> &PublicKey<Proposal> {
        &self.proposal
    }

    fn governing_token_owner(&self) -> &PublicKey<GoverningTokenOwner> {
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
        ctx.approve_vote_choices_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::VoteRecordV2<'a>> for VoteRecord {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::VoteRecordV2 {
            address,
            proposal,
            governing_token_owner,
            is_relinquished,
            voter_weight,
            vote,
            ..
        }: models::VoteRecordV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            proposal: proposal.into_owned().into(),
            governing_token_owner: governing_token_owner.into_owned().into(),
            is_relinquished,
            voter_weight: voter_weight.into(),
            vote: vote.into(),
        })
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

#[derive(Debug, Clone)]
pub struct VoteChoice {
    pub address: PublicKey<VoteRecord>,
    pub rank: i32,
    pub weight_percentage: i32,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Approve Vote VoteChoice")]
impl VoteChoice {
    fn address(&self) -> &PublicKey<VoteRecord> {
        &self.address
    }

    fn rank(&self) -> i32 {
        self.rank
    }

    fn weight_percentage(&self) -> i32 {
        self.weight_percentage
    }
}

impl<'a> TryFrom<models::VoteChoice<'a>> for VoteChoice {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::VoteChoice {
            vote_record_v2_address,
            rank,
            weight_percentage,
            ..
        }: models::VoteChoice,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: vote_record_v2_address.into_owned().into(),
            rank: rank.into(),
            weight_percentage: weight_percentage.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TokenOwnerRecord {
    pub address: PublicKey<TokenOwnerRecord>,
    pub realm: PublicKey<Realm>,
    pub governing_token_mint: PublicKey<TokenMint>,
    pub governing_token_owner: PublicKey<GoverningTokenOwner>,
    pub governing_token_deposit_amount: I64,
    pub unrelinquished_votes_count: I64,
    pub total_votes_count: I64,
    pub outstanding_proposal_count: i32,
    pub governance_delegate: Option<PublicKey<GovernanceDelegate>>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance TokenOwnerRecordV2 account")]
impl TokenOwnerRecord {
    fn address(&self) -> &PublicKey<TokenOwnerRecord> {
        &self.address
    }

    fn realm(&self) -> &PublicKey<Realm> {
        &self.realm
    }

    fn governing_token_mint(&self) -> &PublicKey<TokenMint> {
        &self.governing_token_mint
    }

    fn governing_token_owner(&self) -> &PublicKey<GoverningTokenOwner> {
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
}

impl<'a> TryFrom<models::TokenOwnerRecordV2<'a>> for TokenOwnerRecord {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::TokenOwnerRecordV2 {
            address,
            realm,
            governing_token_mint,
            governing_token_owner,
            governing_token_deposit_amount,
            unrelinquished_votes_count,
            total_votes_count,
            outstanding_proposal_count,
            governance_delegate,
            ..
        }: models::TokenOwnerRecordV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            realm: realm.into_owned().into(),
            governing_token_mint: governing_token_mint.into_owned().into(),
            governing_token_owner: governing_token_owner.into_owned().into(),
            governing_token_deposit_amount: governing_token_deposit_amount.into(),
            unrelinquished_votes_count: unrelinquished_votes_count.into(),
            total_votes_count: total_votes_count.into(),
            outstanding_proposal_count: outstanding_proposal_count.into(),
            governance_delegate: governance_delegate.map(Into::into),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SignatoryRecord {
    pub address: PublicKey<SignatoryRecord>,
    pub proposal: PublicKey<Proposal>,
    pub signatory: PublicKey<Wallet>,
    pub signed_off: bool,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance SignatoryRecordV2 account")]
impl SignatoryRecord {
    fn address(&self) -> &PublicKey<SignatoryRecord> {
        &self.address
    }

    fn proposal(&self) -> &PublicKey<Proposal> {
        &self.proposal
    }

    fn signatory(&self) -> &PublicKey<Wallet> {
        &self.signatory
    }

    fn signed_off(&self) -> bool {
        self.signed_off
    }
}

impl<'a> TryFrom<models::SignatoryRecordV2<'a>> for SignatoryRecord {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::SignatoryRecordV2 {
            address,
            proposal,
            signatory,
            signed_off,
            ..
        }: models::SignatoryRecordV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
            proposal: proposal.into_owned().into(),
            signatory: signatory.into_owned().into(),
            signed_off,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub address: PublicKey<Proposal>,
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
impl Proposal {
    fn address(&self) -> &PublicKey<Proposal> {
        &self.address
    }

    fn governance(&self) -> &PublicKey<Governance> {
        &self.governance
    }

    fn governing_token_mint(&self) -> &PublicKey<TokenMint> {
        &self.governing_token_mint
    }

    fn state(&self) -> &ProposalState {
        &self.state
    }

    fn token_owner_record(&self) -> &PublicKey<TokenOwnerRecord> {
        &self.token_owner_record
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

    // dataloaders
    // vec<proposalOption>
    // voteType multichoice
}

impl<'a> TryFrom<models::ProposalV2<'a>> for Proposal {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::ProposalV2 {
            address,
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
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned().into(),
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
        })
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

#[derive(Debug, Clone)]
pub struct ProposalOption {
    pub proposal_address: PublicKey<Proposal>,
    pub label: String,
    pub vote_weight: I64,
    pub vote_result: OptionVoteResult,
    pub transactions_executed_count: i32,
    pub transactions_count: i32,
    pub transactions_next_index: i32,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "SPLGovernance Proposal option")]
impl ProposalOption {
    fn proposal_address(&self) -> &PublicKey<Proposal> {
        &self.proposal_address
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn vote_weight(&self) -> &I64 {
        &self.vote_weight
    }

    fn vote_result(&self) -> &OptionVoteResult {
        &self.vote_result
    }

    fn transactions_executed_count(&self) -> i32 {
        self.transactions_executed_count
    }

    fn transactions_count(&self) -> i32 {
        self.transactions_count
    }

    fn transactions_next_index(&self) -> i32 {
        self.transactions_next_index
    }
}

impl<'a> TryFrom<models::ProposalOption<'a>> for ProposalOption {
    type Error = std::num::TryFromIntError;
    fn try_from(
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
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_address: proposal_address.into_owned().into(),
            label: label.into_owned(),
            vote_weight: vote_weight.into(),
            vote_result: vote_result.into(),
            transactions_executed_count: transactions_executed_count.into(),
            transactions_count: transactions_count.into(),
            transactions_next_index: transactions_next_index.into(),
        })
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
