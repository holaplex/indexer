//! Query utilities for SPL GOVERNANCE

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Bool, Nullable, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{
        models::{SplGovernanceProposal, VoteRecord},
        Connection,
    },
    error::Result,
};

const VOTE_RECORDS_QUERY: &str = r"
select 	address, account_type, proposal, governing_token_owner, is_relinquished, null as voter_weight,
		null as vote, vote_type, vote_weight, slot, write_version
from vote_records_v1
where address = any($1)
	and (proposal = any($2) or $2 is null)
	and (governing_token_owner = any($3) or $3 is null)
	and (is_relinquished = $4 or $4 is null)
union all
select 	address, account_type, proposal, governing_token_owner, is_relinquished, voter_weight,
		vote, null as vote_type, null as vote_weight, slot, write_version
from vote_records_v2
where (address = any($1) or $1 is null)
    and (proposal = any($2) or $2 is null)
    and (governing_token_owner = any($3) or $3 is null)
    and (is_relinquished = $4 or $4 is null);
 -- $1: addresses::text[]
 -- $2: proposals::text[]
 -- $3: governing_token_owners::text[]
 -- $4: is_relinquished::bool";

/// Load all spl governance vote records including V1 and V2
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn vote_records(
    conn: &Connection,
    addresses: impl ToSql<Nullable<Array<Text>>, Pg>,
    proposals: impl ToSql<Nullable<Array<Text>>, Pg>,
    governing_token_owners: impl ToSql<Nullable<Array<Text>>, Pg>,
    is_relinquished: impl ToSql<Nullable<Bool>, Pg>,
) -> Result<Vec<VoteRecord>> {
    diesel::sql_query(VOTE_RECORDS_QUERY)
        .bind(addresses)
        .bind(proposals)
        .bind(governing_token_owners)
        .bind(is_relinquished)
        .load(conn)
        .context("Failed to load vote records")
}

const PROPOSALS_QUERY: &str = r"
select 	address, account_type, governance, governing_token_mint, state, token_owner_record, signatories_count,
		signatories_signed_off_count, yes_votes_count, no_votes_count, instructions_executed_count,
		instructions_count, instructions_next_index, null as vote_type, null as deny_vote_weight, null as veto_vote_weight,
		null as abstain_vote_weight, null as start_voting_at, draft_at, signing_off_at, voting_at, null as voting_at_slot,
		voting_completed_at, executing_at, closed_at, execution_flags, max_vote_weight, null as max_voting_time,
		vote_threshold_type, vote_threshold_percentage, name, description_link
from proposals_v1
where (address = any($1) or $1 is null) and (governance = any($2) or $2 is null)
union all
select 	address, account_type, governance, governing_token_mint, state, token_owner_record, signatories_count,
		signatories_signed_off_count, null as yes_votes_count, null as no_votes_count, null as instructions_executed_count,
		null as instructions_count, null as instructions_next_index, vote_type, deny_vote_weight, veto_vote_weight, abstain_vote_weight,
		start_voting_at, draft_at, signing_off_at, voting_at, voting_at_slot, voting_completed_at, executing_at, closed_at, execution_flags,
		max_vote_weight, max_voting_time, vote_threshold_type, vote_threshold_percentage, name, description_link
from proposals_v2
where (address = any($1) or $1 is null) and (governance = any($2) or $2 is null);
 -- $1: addresses::text[]
 -- $2: governances::text[]";

/// Load all spl governance proposals including V1 and V2
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn proposals(
    conn: &Connection,
    addresses: impl ToSql<Nullable<Array<Text>>, Pg>,
    governances: impl ToSql<Nullable<Array<Text>>, Pg>,
) -> Result<Vec<SplGovernanceProposal>> {
    diesel::sql_query(PROPOSALS_QUERY)
        .bind(addresses)
        .bind(governances)
        .load(conn)
        .context("Failed to load proposals")
}
