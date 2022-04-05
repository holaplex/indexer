use anchor_lang_v0_22_1::AccountDeserialize;
use tribeca_govern::{Governor, Proposal, ProposalMeta, Vote};

use super::{accounts::govern, AccountUpdate, Client};
use crate::prelude::*;

const GOVERNOR_SIZE: usize = 8 + 32 + 1 + 8 + 32 + 32 + GOVERNANCE_PARAMS_SIZE;
const GOVERNANCE_PARAMS_SIZE: usize = 8 + 8 + 8 + 8;
const VOTE_SIZE: usize = 8 + 32 + 32 + 1 + 1 + 8;

async fn process_governor(client: &Client, update: AccountUpdate) -> Result<()> {
    let governor = Governor::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize governor account!")?;

    govern::process_governor(client, update.key, governor).await
}

async fn process_vote(client: &Client, update: AccountUpdate) -> Result<()> {
    let vote = Vote::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize vote account!")?;

    govern::process_vote(client, update.key, vote).await
}

async fn process_proposal_and_meta(client: &Client, update: AccountUpdate) -> Result<()> {
    if let Ok(proposal) = Proposal::try_deserialize_unchecked(&mut update.data.as_slice()) {
        govern::process_proposal(client, update.key, proposal).await?;
    } else {
        let proposal_meta = ProposalMeta::try_deserialize_unchecked(&mut update.data.as_slice())
            .context("failed to deserialize proposal meta account!")?;

        govern::process_meta(client, update.key, proposal_meta).await?;
    }

    Ok(())
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        GOVERNOR_SIZE => process_governor(client, update).await,
        VOTE_SIZE => process_vote(client, update).await,
        _ => process_proposal_and_meta(client, update).await,
    }
}
