//! Tribeca Govern program accounts indexing
use indexer_core::{
    db::{
        insert_into,
        models::{
            GovernanceParameter as GovernanceParameterModel, Governor as GovernorModel,
            Proposal as ProposalModel, ProposalAccountMeta as ProposalAccountMetaModel,
            ProposalInstruction as ProposalInstructionModel, ProposalMeta as ProposalMetaModel,
            Vote as VoteModel,
        },
        tables::{
            governance_parameters, governors, proposal_account_metas, proposal_instructions,
            proposal_metas, proposals, votes,
        },
    },
    prelude::*,
};
use tribeca_govern::{
    GovernanceParameters, Governor, Proposal, ProposalAccountMeta, ProposalInstruction,
    ProposalMeta, Vote,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_governor(client: &Client, key: Pubkey, g: Governor) -> Result<()> {
    let row = GovernorModel {
        address: Owned(key.to_string()),
        base: Owned(g.base.to_string()),
        bump: g.bump.try_into()?,
        proposal_count: g.proposal_count.try_into()?,
        electorate: Owned(g.electorate.to_string()),
        smart_wallet: Owned(g.smart_wallet.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(governors::table)
                .values(&row)
                .on_conflict(governors::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert governor ")?;

    process_governance_params(client, key, g.params).await
}

async fn process_governance_params(
    client: &Client,
    key: Pubkey,
    param: GovernanceParameters,
) -> Result<()> {
    let row = GovernanceParameterModel {
        governor_address: Owned(key.to_string()),
        voting_delay: param.voting_delay.try_into()?,
        voting_period: param.voting_period.try_into()?,
        quorum_votes: param.quorum_votes.try_into()?,
        timelock_delay_seconds: param.timelock_delay_seconds,
    };

    client
        .db()
        .run(move |db| {
            insert_into(governance_parameters::table)
                .values(&row)
                .on_conflict(governance_parameters::governor_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert governance parameters ")?;

    Ok(())
}

pub(crate) async fn process_vote(client: &Client, key: Pubkey, v: Vote) -> Result<()> {
    let row = VoteModel {
        address: Owned(key.to_string()),
        proposal: Owned(v.proposal.to_string()),
        voter: Owned(v.voter.to_string()),
        bump: v.bump.try_into()?,
        side: v.side.try_into()?,
        weight: v.weight.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(votes::table)
                .values(&row)
                .on_conflict(votes::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert vote ")?;

    Ok(())
}

pub(crate) async fn process_meta(client: &Client, key: Pubkey, meta: ProposalMeta) -> Result<()> {
    let row = ProposalMetaModel {
        address: Owned(key.to_string()),
        proposal: Owned(meta.proposal.to_string()),
        title: Owned(meta.title),
        description_link: Owned(meta.description_link),
    };

    client
        .db()
        .run(move |db| {
            insert_into(proposal_metas::table)
                .values(&row)
                .on_conflict(proposal_metas::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert proposal metadata ")?;

    Ok(())
}

pub(crate) async fn process_proposal(
    client: &Client,
    key: Pubkey,
    proposal: Proposal,
) -> Result<()> {
    let row = ProposalModel {
        address: Owned(key.to_string()),
        governor: Owned(proposal.governor.to_string()),
        index: proposal.index.try_into()?,
        bump: proposal.bump.try_into()?,
        proposer: Owned(proposal.proposer.to_string()),
        quorum_votes: proposal.quorum_votes.try_into()?,
        for_votes: proposal.for_votes.try_into()?,
        against_votes: proposal.against_votes.try_into()?,
        abstain_votes: proposal.abstain_votes.try_into()?,
        canceled_at: proposal.canceled_at,
        created_at: proposal.created_at,
        activated_at: proposal.activated_at,
        voting_ends_at: proposal.voting_ends_at,
        queued_at: proposal.queued_at,
        queued_transaction: Owned(proposal.queued_transaction.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(proposals::table)
                .values(&row)
                .on_conflict(proposals::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert proposal ")?;

    process_instructions(client, key, proposal.instructions).await
}

pub(crate) async fn process_instructions(
    client: &Client,
    key: Pubkey,
    instructions: Vec<ProposalInstruction>,
) -> Result<()> {
    for ins in instructions {
        let i = ProposalInstructionModel {
            proposal_address: Owned(key.to_string()),
            program_id: Owned(ins.program_id.to_string()),
            data: ins.data,
        };

        client
            .db()
            .run(move |db| {
                insert_into(proposal_instructions::table)
                    .values(&i)
                    .on_conflict((
                        proposal_instructions::proposal_address,
                        proposal_instructions::program_id,
                    ))
                    .do_update()
                    .set(&i)
                    .execute(db)
            })
            .await
            .context("failed to insert proposal instruction ")?;
        process_account_meta(client, key, ins.program_id, ins.keys).await?;
    }

    Ok(())
}

async fn process_account_meta(
    client: &Client,
    key: Pubkey,
    program_id: Pubkey,
    account_metas: Vec<ProposalAccountMeta>,
) -> Result<()> {
    for acc in account_metas {
        let row = ProposalAccountMetaModel {
            proposal_address: Owned(key.to_string()),
            program_id: Owned(program_id.to_string()),
            pubkey: Owned(acc.pubkey.to_string()),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        };

        client
            .db()
            .run(move |db| {
                insert_into(proposal_account_metas::table)
                    .values(&row)
                    .on_conflict((
                        proposal_account_metas::proposal_address,
                        proposal_account_metas::program_id,
                        proposal_account_metas::pubkey,
                    ))
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("failed to insert proposal account metadata ")?;
    }
    Ok(())
}
