use borsh::BorshDeserialize;

use super::{
    accounts::spl_governance::{
        process_governance, process_proposal_transaction, process_proposal_v2, process_realmv2,
        process_signatory_record_v2, process_token_owner_record_v2, process_vote_record_v2,
        GovernanceAccountType, GovernanceV2, ProposalTransactionV2, ProposalV2, RealmV2,
        SignatoryRecordV2, TokenOwnerRecordV2, VoteRecordV2,
    },
    AccountUpdate, Client,
};
use crate::prelude::*;

const GOVERNANCE_V2: u8 = GovernanceAccountType::GovernanceV2 as u8;
const REALM_V2: u8 = GovernanceAccountType::RealmV2 as u8;
const VOTE_RECORD_V2: u8 = GovernanceAccountType::VoteRecordV2 as u8;
const TOKEN_OWNER_RECORD_V2: u8 = GovernanceAccountType::TokenOwnerRecordV2 as u8;
const PROPOSAL_V2: u8 = GovernanceAccountType::ProposalV2 as u8;
const SIGNATORY_RECORD_V2: u8 = GovernanceAccountType::SignatoryRecordV2 as u8;
const PROPOSAL_TRANSACTION_V2: u8 = GovernanceAccountType::ProposalTransactionV2 as u8;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let discrimintator = update.data[0];

    match discrimintator {
        GOVERNANCE_V2 => process_governancev2_account(client, update).await,
        REALM_V2 => process_realmv2_account(client, update).await,
        VOTE_RECORD_V2 => process_vote_recordv2_account(client, update).await,
        TOKEN_OWNER_RECORD_V2 => process_token_owner_record_v2_account(client, update).await,
        PROPOSAL_V2 => process_proposalv2_account(client, update).await,
        SIGNATORY_RECORD_V2 => process_signatory_recordv2_account(client, update).await,
        PROPOSAL_TRANSACTION_V2 => process_proposal_transaction_account(client, update).await,
        _ => Ok(()),
    }
}

async fn process_governancev2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: GovernanceV2 = GovernanceV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize spl governance v2 account ")?;

    process_governance(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_realmv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: RealmV2 = RealmV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize spl realm v2 account ")?;

    process_realmv2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_vote_recordv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: VoteRecordV2 = VoteRecordV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize vote record v2 ")?;

    process_vote_record_v2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_token_owner_record_v2_account(
    client: &Client,
    update: AccountUpdate,
) -> Result<()> {
    let acc: TokenOwnerRecordV2 = TokenOwnerRecordV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize token owner record v2 account ")?;

    process_token_owner_record_v2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_proposalv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: ProposalV2 = ProposalV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize proposal v2 account")?;

    process_proposal_v2(client, update.key, acc, update.slot, update.write_version).await
}

async fn process_signatory_recordv2_account(client: &Client, update: AccountUpdate) -> Result<()> {
    let acc: SignatoryRecordV2 = SignatoryRecordV2::deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize signatory record v2 account ")?;

    process_signatory_record_v2(client, update.key, acc, update.slot, update.write_version).await
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
