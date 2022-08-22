use borsh::BorshDeserialize;

use super::{
    accounts::spl_governance::{
        process_governance, process_proposal_transaction, process_proposal_v1, process_proposal_v2,
        process_realmv2, process_signatory_record, process_token_owner_record,
        process_vote_record_v1, process_vote_record_v2, GovernanceAccountType, GovernanceV1,
        ProposalTransactionV2, ProposalV1, ProposalV2, RealmV1, SignatoryRecordV1,
        TokenOwnerRecordV1, VoteRecordV1, VoteRecordV2,
    },
    AccountUpdate, Client,
};
use crate::prelude::*;

const GOVERNANCE_V1: u8 = GovernanceAccountType::GovernanceV1 as u8;
const GOVERNANCE_V2: u8 = GovernanceAccountType::GovernanceV2 as u8;
const REALM_V1: u8 = GovernanceAccountType::RealmV1 as u8;
const REALM_V2: u8 = GovernanceAccountType::RealmV2 as u8;
const VOTE_RECORD_V1: u8 = GovernanceAccountType::VoteRecordV1 as u8;
const VOTE_RECORD_V2: u8 = GovernanceAccountType::VoteRecordV2 as u8;
const TOKEN_OWNER_RECORD_V1: u8 = GovernanceAccountType::TokenOwnerRecordV1 as u8;
const TOKEN_OWNER_RECORD_V2: u8 = GovernanceAccountType::TokenOwnerRecordV2 as u8;
const PROPOSAL_V1: u8 = GovernanceAccountType::ProposalV1 as u8;
const PROPOSAL_V2: u8 = GovernanceAccountType::ProposalV2 as u8;
const SIGNATORY_RECORD_V1: u8 = GovernanceAccountType::SignatoryRecordV1 as u8;
const SIGNATORY_RECORD_V2: u8 = GovernanceAccountType::SignatoryRecordV2 as u8;
const PROPOSAL_TRANSACTION_V2: u8 = GovernanceAccountType::ProposalTransactionV2 as u8;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let discrimintator = update.data[0];

    match discrimintator {
        GOVERNANCE_V1 | GOVERNANCE_V2 => process_governance_account(client, update).await,
        REALM_V1 | REALM_V2 => process_realm_account(client, update).await,
        VOTE_RECORD_V1 => process_vote_recordv1_account(client, update).await,
        VOTE_RECORD_V2 => process_vote_recordv2_account(client, update).await,
        TOKEN_OWNER_RECORD_V1 | TOKEN_OWNER_RECORD_V2 => {
            process_token_owner_record_account(client, update).await
        },
        PROPOSAL_V1 => process_proposalv1_account(client, update).await,
        PROPOSAL_V2 => process_proposalv2_account(client, update).await,
        SIGNATORY_RECORD_V1 | SIGNATORY_RECORD_V2 => {
            process_signatory_record_account(client, update).await
        },
        PROPOSAL_TRANSACTION_V2 => process_proposal_transaction_account(client, update).await,
        _ => Ok(()),
    }
}

async fn process_governance_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: GovernanceV1 = GovernanceV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize spl governance  account ")?;

    process_governance(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_realm_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: RealmV1 = RealmV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize spl realm account ")?;

    process_realmv2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_vote_recordv1_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: VoteRecordV1 = VoteRecordV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize vote record v1 account ")?;

    process_vote_record_v1(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_vote_recordv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: VoteRecordV2 = VoteRecordV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize vote record v2 ")?;

    process_vote_record_v2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_token_owner_record_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: TokenOwnerRecordV1 = TokenOwnerRecordV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize token owner record account ")?;

    process_token_owner_record(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_proposalv1_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: ProposalV1 = ProposalV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize proposal v1 account")?;

    process_proposal_v1(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_proposalv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: ProposalV2 = ProposalV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize proposal v2 account")?;

    process_proposal_v2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_signatory_record_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: SignatoryRecordV1 = SignatoryRecordV1::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize signatory record v2 account ")?;

    process_signatory_record(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_proposal_transaction_account(
    client: &Client,
    update: AccountUpdate,
) -> Result<()> {
    let acc: ProposalTransactionV2 =
        ProposalTransactionV2::deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize spl governance proposal transaction account")?;

    process_proposal_transaction(client, update.key, acc, update.slot, update.write_version).await
}
