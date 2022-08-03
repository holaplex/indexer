use borsh::{BorshDeserialize, BorshSerialize};
use indexer_core::{
    db::{
        custom_types::{
            GovernanceAccountTypeEnum, InstructionExecutionFlagsEnum, MintMaxVoteEnum,
            OptionVoteResultEnum, ProposalStateEnum, ProposalVoteTypeEnum, VoteRecordV2VoteEnum,
            VoteThresholdEnum, VoteTippingEnum,
        },
        insert_into,
        models::{
            Governance, GovernanceConfig as DbGovernanceConfig, MultiChoice,
            ProposalOption as DbProposalOption, ProposalV2 as DbProposalV2, Realm,
            RealmConfig as DbRealmConfig, SignatoryRecordV2 as DbSignatoryRecordV2,
            TokenOwnerRecordV2 as DbTokenOwnerRecordV2, VoteChoice as DbVoteChoice,
            VoteRecordV2 as DbVoteRecordV2,
        },
        tables::{
            governance_configs, governances, proposal_options, proposal_vote_type_multi_choices,
            proposals_v2, realm_configs, realms, signatory_records_v2, token_owner_records_v2,
            vote_record_v2_vote_approve_vote_choices, vote_records_v2,
        },
    },
    prelude::*,
    util::unix_timestamp,
};
use solana_program::{clock::UnixTimestamp, slot_history::Slot};

use super::Client;
use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
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

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum MintMaxVoteWeightSource {
    SupplyFraction(u64),
    Absolute(u64),
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum VoteThresholdPercentage {
    YesVote(u8),
    Quorum(u8),
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum VoteTipping {
    Strict,
    Early,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct GovernanceV2 {
    pub account_type: GovernanceAccountType,
    pub realm: Pubkey,
    pub governed_account: Pubkey,
    pub proposals_count: u32,
    pub config: GovernanceConfig,
    pub reserved: [u8; 6],
    pub voting_proposal_count: u16,
    pub reserved_v2: [u8; 128],
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct GovernanceConfig {
    pub vote_threshold_percentage: VoteThresholdPercentage,
    pub min_community_weight_to_create_proposal: u64,
    pub min_transaction_hold_up_time: u32,
    pub max_voting_time: u32,
    pub vote_tipping: VoteTipping,
    pub proposal_cool_off_time: u32,
    pub min_council_weight_to_create_proposal: u64,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct RealmV2 {
    pub account_type: GovernanceAccountType,
    pub community_mint: Pubkey,
    pub config: RealmConfig,
    pub reserved: [u8; 6],
    pub voting_proposal_count: u16,
    pub authority: Option<Pubkey>,
    pub name: String,
    pub reserved_v2: [u8; 128],
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct RealmConfig {
    pub use_community_voter_weight_addin: bool,
    pub use_max_community_voter_weight_addin: bool,
    pub reserved: [u8; 6],
    pub min_community_weight_to_create_governance: u64,
    pub community_mint_max_vote_weight_source: MintMaxVoteWeightSource,
    pub council_mint: Option<Pubkey>,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct VoteRecordV2 {
    pub account_type: GovernanceAccountType,
    pub proposal: Pubkey,
    pub governing_token_owner: Pubkey,
    pub is_relinquished: bool,
    pub voter_weight: u64,
    pub vote: Vote,
    pub reserved_v2: [u8; 8],
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Vote {
    Approve(Vec<VoteChoice>),
    Deny,
    Abstain,
    Veto,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct VoteChoice {
    pub rank: u8,
    pub weight_percentage: u8,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct TokenOwnerRecordV2 {
    pub account_type: GovernanceAccountType,
    pub realm: Pubkey,
    pub governing_token_mint: Pubkey,
    pub governing_token_owner: Pubkey,
    pub governing_token_deposit_amount: u64,
    pub unrelinquished_votes_count: u32,
    pub total_votes_count: u32,
    pub outstanding_proposal_count: u8,
    pub reserved: [u8; 7],
    pub governance_delegate: Option<Pubkey>,
    pub reserved_v2: [u8; 128],
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct SignatoryRecordV2 {
    pub account_type: GovernanceAccountType,
    pub proposal: Pubkey,
    pub signatory: Pubkey,
    pub signed_off: bool,
    pub reserved_v2: [u8; 8],
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct ProposalOption {
    pub label: String,
    pub vote_weight: u64,
    pub vote_result: OptionVoteResult,
    pub transactions_executed_count: u16,
    pub transactions_count: u16,
    pub transactions_next_index: u16,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct ProposalV2 {
    pub account_type: GovernanceAccountType,
    pub governance: Pubkey,
    pub governing_token_mint: Pubkey,
    pub state: ProposalState,
    pub token_owner_record: Pubkey,
    pub signatories_count: u8,
    pub signatories_signed_off_count: u8,
    pub vote_type: VoteType,
    pub options: Vec<ProposalOption>,
    pub deny_vote_weight: Option<u64>,
    pub veto_vote_weight: Option<u64>,
    pub abstain_vote_weight: Option<u64>,
    pub start_voting_at: Option<UnixTimestamp>,
    pub draft_at: UnixTimestamp,
    pub signing_off_at: Option<UnixTimestamp>,
    pub voting_at: Option<UnixTimestamp>,
    pub voting_at_slot: Option<Slot>,
    pub voting_completed_at: Option<UnixTimestamp>,
    pub executing_at: Option<UnixTimestamp>,
    pub closed_at: Option<UnixTimestamp>,
    pub execution_flags: InstructionExecutionFlags,
    pub max_vote_weight: Option<u64>,
    pub max_voting_time: Option<u32>,
    pub vote_threshold_percentage: Option<VoteThresholdPercentage>,
    pub reserved: [u8; 64],
    pub name: String,
    pub description_link: String,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum OptionVoteResult {
    None,
    Succeeded,
    Defeated,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum VoteType {
    SingleChoice,
    MultiChoice {
        max_voter_options: u8,
        max_winning_options: u8,
    },
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
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

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum InstructionExecutionFlags {
    None,
    Ordered,
    UseTransaction,
}

pub(crate) async fn process_governance(
    client: &Client,
    key: Pubkey,
    data: GovernanceV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = Governance {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::GovernanceV1 => GovernanceAccountTypeEnum::GovernanceV1,
            GovernanceAccountType::GovernanceV2 => GovernanceAccountTypeEnum::GovernanceV2,
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        realm: Owned(data.realm.to_string()),
        governed_account: Owned(data.governed_account.to_string()),
        proposals_count: data.proposals_count.try_into()?,
        reserved: Owned(data.reserved.to_vec()),
        voting_proposal_count: data.voting_proposal_count.try_into()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(governances::table)
                .values(&row)
                .on_conflict(governances::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert governance account")?;

    let c = data.config;
    let (vote_threshold_type, vote_threshold_percentage) = match c.vote_threshold_percentage {
        VoteThresholdPercentage::YesVote(p) => (VoteThresholdEnum::YesVote, i16::try_from(p)?),
        VoteThresholdPercentage::Quorum(p) => (VoteThresholdEnum::Quorum, i16::try_from(p)?),
    };

    let config = DbGovernanceConfig {
        governance_address: Owned(key.to_string()),
        vote_threshold_type,
        vote_threshold_percentage,
        min_community_weight_to_create_proposal: c
            .min_community_weight_to_create_proposal
            .try_into()?,
        min_instruction_hold_up_time: c.min_transaction_hold_up_time.try_into()?,
        max_voting_time: c.max_voting_time.try_into()?,
        vote_tipping: match c.vote_tipping {
            VoteTipping::Strict => VoteTippingEnum::Strict,
            VoteTipping::Early => VoteTippingEnum::Early,
            VoteTipping::Disabled => VoteTippingEnum::Disabled,
        },
        proposal_cool_off_time: c.proposal_cool_off_time.try_into()?,
        min_council_weight_to_create_proposal: c
            .min_council_weight_to_create_proposal
            .try_into()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(governance_configs::table)
                .values(&config)
                .on_conflict(governance_configs::governance_address)
                .do_update()
                .set(&config)
                .execute(db)
        })
        .await
        .context("Failed to insert governance config")?;

    Ok(())
}

pub(crate) async fn process_realmv2(
    client: &Client,
    key: Pubkey,
    data: RealmV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = Realm {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::RealmV1 => GovernanceAccountTypeEnum::RealmV1,
            GovernanceAccountType::RealmV2 => GovernanceAccountTypeEnum::RealmV2,
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        community_mint: Owned(data.community_mint.to_string()),
        reserved: Owned(data.reserved.to_vec()),
        voting_proposal_count: data.voting_proposal_count.try_into()?,
        authority: data.authority.map(|a| Owned(a.to_string())),
        name: Owned(data.name.to_string()),
        reserved_v2: Owned(data.reserved_v2.to_vec()),
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(realms::table)
                .values(&row)
                .on_conflict(realms::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert realm account")?;

    let c = data.config;
    let (vote_weight_source, vote_weight) = match c.community_mint_max_vote_weight_source {
        MintMaxVoteWeightSource::SupplyFraction(percentage) => {
            (MintMaxVoteEnum::SupplyFraction, percentage)
        },
        MintMaxVoteWeightSource::Absolute(percentage) => (MintMaxVoteEnum::Absolute, percentage),
    };

    let config = DbRealmConfig {
        realm_address: Owned(key.to_string()),
        use_community_voter_weight_addin: c.use_community_voter_weight_addin,
        use_max_community_voter_weight_addin: c.use_max_community_voter_weight_addin,
        reserved: Owned(c.reserved.to_vec()),
        min_community_weight_to_create_governance: c
            .min_community_weight_to_create_governance
            .try_into()?,
        community_mint_max_vote_weight_source: vote_weight_source,
        community_mint_max_vote_weight: vote_weight.try_into()?,
        council_mint: c.council_mint.map(|c| Owned(c.to_string())),
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(realm_configs::table)
                .values(&config)
                .on_conflict(realm_configs::realm_address)
                .do_update()
                .set(&config)
                .execute(db)
        })
        .await
        .context("Failed to insert realm config")?;

    Ok(())
}

pub(crate) async fn process_vote_record_v2(
    client: &Client,
    key: Pubkey,
    data: VoteRecordV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbVoteRecordV2 {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::VoteRecordV2 => GovernanceAccountTypeEnum::VoteRecordV2,
            GovernanceAccountType::VoteRecordV1 => GovernanceAccountTypeEnum::VoteRecordV1,
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        proposal: Owned(data.proposal.to_string()),
        governing_token_owner: Owned(data.governing_token_owner.to_string()),
        is_relinquished: data.is_relinquished,
        voter_weight: data.voter_weight.try_into()?,
        vote: match data.vote {
            Vote::Approve(_) => VoteRecordV2VoteEnum::Approve,
            Vote::Deny => VoteRecordV2VoteEnum::Deny,
            Vote::Abstain => VoteRecordV2VoteEnum::Abstain,
            Vote::Veto => VoteRecordV2VoteEnum::Veto,
        },
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(vote_records_v2::table)
                .values(&row)
                .on_conflict(vote_records_v2::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert vote record v2")?;

    if let Vote::Approve(choices) = data.vote {
        for c in choices {
            let r = DbVoteChoice {
                vote_record_v2_address: Owned(key.to_string()),
                rank: c.rank.try_into()?,
                weight_percentage: c.weight_percentage.try_into()?,
                slot: slot.try_into()?,
                write_version: write_version.try_into()?,
            };

            client
                .db()
                .run(move |db| {
                    insert_into(vote_record_v2_vote_approve_vote_choices::table)
                        .values(&r)
                        .on_conflict((
                            vote_record_v2_vote_approve_vote_choices::vote_record_v2_address,
                            vote_record_v2_vote_approve_vote_choices::rank,
                            vote_record_v2_vote_approve_vote_choices::weight_percentage,
                        ))
                        .do_update()
                        .set(&r)
                        .execute(db)
                })
                .await
                .context("Failed to insert vote record v2 approve vote choice")?;
        }
    }

    Ok(())
}

pub(crate) async fn process_token_owner_record_v2(
    client: &Client,
    key: Pubkey,
    data: TokenOwnerRecordV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbTokenOwnerRecordV2 {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::TokenOwnerRecordV2 => {
                GovernanceAccountTypeEnum::TokenOwnerRecordV2
            },
            GovernanceAccountType::TokenOwnerRecordV1 => {
                GovernanceAccountTypeEnum::TokenOwnerRecordV1
            },
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        realm: Owned(data.realm.to_string()),
        governing_token_mint: Owned(data.governing_token_mint.to_string()),
        governing_token_owner: Owned(data.governing_token_owner.to_string()),
        governing_token_deposit_amount: data.governing_token_deposit_amount.try_into()?,
        unrelinquished_votes_count: data.unrelinquished_votes_count.try_into()?,
        total_votes_count: data.total_votes_count.try_into()?,
        outstanding_proposal_count: data.outstanding_proposal_count.try_into()?,
        reserved: Owned(data.reserved.to_vec()),
        governance_delegate: data.governance_delegate.map(|d| Owned(d.to_string())),
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(token_owner_records_v2::table)
                .values(&row)
                .on_conflict(token_owner_records_v2::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert vote record v2")?;

    Ok(())
}

pub(crate) async fn process_signatory_record_v2(
    client: &Client,
    key: Pubkey,
    data: SignatoryRecordV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbSignatoryRecordV2 {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::SignatoryRecordV2 => {
                GovernanceAccountTypeEnum::SignatoryRecordV2
            },
            GovernanceAccountType::SignatoryRecordV1 => {
                GovernanceAccountTypeEnum::SignatoryRecordV1
            },
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        proposal: Owned(data.proposal.to_string()),
        signatory: Owned(data.signatory.to_string()),
        signed_off: data.signed_off,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(signatory_records_v2::table)
                .values(&row)
                .on_conflict(signatory_records_v2::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert signatory record v2")?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn process_proposal_v2(
    client: &Client,
    key: Pubkey,
    data: ProposalV2,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let (vote_threshold_type, vote_threshold_percentage) =
        if let Some(VoteThresholdPercentage::YesVote(p)) = data.vote_threshold_percentage {
            (Some(VoteThresholdEnum::YesVote), Some(i16::try_from(p)?))
        } else if let Some(VoteThresholdPercentage::Quorum(p)) = data.vote_threshold_percentage {
            (Some(VoteThresholdEnum::Quorum), Some(i16::try_from(p)?))
        } else {
            (None, None)
        };

    let row = DbProposalV2 {
        address: Owned(key.to_string()),
        account_type: match data.account_type {
            GovernanceAccountType::ProposalV2 => GovernanceAccountTypeEnum::ProposalV2,
            GovernanceAccountType::ProposalV1 => GovernanceAccountTypeEnum::ProposalV1,
            _ => GovernanceAccountTypeEnum::Uninitialized,
        },
        governance: Owned(data.governance.to_string()),
        governing_token_mint: Owned(data.governing_token_mint.to_string()),
        state: match data.state {
            ProposalState::Draft => ProposalStateEnum::Draft,
            ProposalState::SigningOff => ProposalStateEnum::SigningOff,
            ProposalState::Voting => ProposalStateEnum::Voting,
            ProposalState::Succeeded => ProposalStateEnum::Succeeded,
            ProposalState::Executing => ProposalStateEnum::Executing,
            ProposalState::Completed => ProposalStateEnum::Completed,
            ProposalState::Cancelled => ProposalStateEnum::Cancelled,
            ProposalState::Defeated => ProposalStateEnum::Defeated,
            ProposalState::ExecutingWithErrors => ProposalStateEnum::ExecutingWithErrors,
        },
        token_owner_record: Owned(data.token_owner_record.to_string()),
        signatories_count: data.signatories_count.try_into()?,
        signatories_signed_off_count: data.signatories_signed_off_count.try_into()?,
        vote_type: match data.vote_type {
            VoteType::SingleChoice => ProposalVoteTypeEnum::SingleChoice,
            VoteType::MultiChoice { .. } => ProposalVoteTypeEnum::MultiChoice,
        },
        deny_vote_weight: data.deny_vote_weight.map(TryInto::try_into).transpose()?,
        veto_vote_weight: data.veto_vote_weight.map(TryInto::try_into).transpose()?,

        abstain_vote_weight: data
            .abstain_vote_weight
            .map(TryInto::try_into)
            .transpose()?,
        start_voting_at: data.start_voting_at.map(unix_timestamp).transpose()?,
        draft_at: unix_timestamp(data.draft_at)?,
        signing_off_at: data.signing_off_at.map(unix_timestamp).transpose()?,
        voting_at: data.voting_at.map(unix_timestamp).transpose()?,
        voting_at_slot: data.voting_at_slot.map(TryInto::try_into).transpose()?,
        voting_completed_at: data.voting_completed_at.map(unix_timestamp).transpose()?,
        executing_at: data.executing_at.map(unix_timestamp).transpose()?,
        closed_at: data.closed_at.map(unix_timestamp).transpose()?,
        execution_flags: match data.execution_flags {
            InstructionExecutionFlags::None => InstructionExecutionFlagsEnum::None,
            InstructionExecutionFlags::Ordered => InstructionExecutionFlagsEnum::Ordered,
            InstructionExecutionFlags::UseTransaction => {
                InstructionExecutionFlagsEnum::UseTransaction
            },
        },
        max_vote_weight: data.max_vote_weight.map(TryInto::try_into).transpose()?,
        max_voting_time: data.max_voting_time.map(TryInto::try_into).transpose()?,
        vote_threshold_type,
        vote_threshold_percentage,
        name: Owned(data.name.to_string()),
        description_link: Owned(data.description_link.to_string()),
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(proposals_v2::table)
                .values(&row)
                .on_conflict(proposals_v2::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert proposal v2")?;

    for o in data.options {
        let row = DbProposalOption {
            proposal_address: Owned(key.to_string()),
            label: Owned(o.label.to_string()),
            vote_weight: o.vote_weight.try_into()?,
            vote_result: match o.vote_result {
                OptionVoteResult::None => OptionVoteResultEnum::None,
                OptionVoteResult::Succeeded => OptionVoteResultEnum::Succeeded,
                OptionVoteResult::Defeated => OptionVoteResultEnum::Defeated,
            },
            transactions_executed_count: o.transactions_next_index.try_into()?,
            transactions_count: o.transactions_count.try_into()?,
            transactions_next_index: o.transactions_next_index.try_into()?,
            slot: slot.try_into()?,
            write_version: write_version.try_into()?,
        };

        client
            .db()
            .run(move |db| {
                insert_into(proposal_options::table)
                    .values(&row)
                    .on_conflict((proposal_options::proposal_address, proposal_options::label))
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("Failed to insert proposal option")?;
    }

    if let VoteType::MultiChoice {
        max_voter_options,
        max_winning_options,
    } = data.vote_type
    {
        let row = MultiChoice {
            address: Owned(key.to_string()),
            max_voter_options: max_voter_options.try_into()?,
            max_winning_options: max_winning_options.try_into()?,
            slot: slot.try_into()?,
            write_version: write_version.try_into()?,
        };

        client
            .db()
            .run(move |db| {
                insert_into(proposal_vote_type_multi_choices::table)
                    .values(&row)
                    .on_conflict(proposal_vote_type_multi_choices::address)
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("Failed to insert multichoice vote type")?;
    }

    Ok(())
}
