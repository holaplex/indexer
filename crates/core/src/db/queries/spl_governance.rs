//! Query utilities for NFT activity.

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
SELECT ADDRESS,
	ACCOUNT_TYPE,
	PROPOSAL,
	GOVERNING_TOKEN_OWNER,
	IS_RELINQUISHED,
	NULL as voter_weight,
	NULL as vote,
	VOTE_TYPE,
	VOTE_WEIGHT,
	SLOT,
	WRITE_VERSION
FROM VOTE_RECORDS_V1
WHERE ADDRESS = ANY($1)
	AND (PROPOSAL = ANY($2) OR $2 is null)
	AND (GOVERNING_TOKEN_OWNER = ANY($3) OR $3 is null)
	AND (is_relinquished = $4 OR $4 is null)
UNION ALL
SELECT ADDRESS,
	ACCOUNT_TYPE,
	PROPOSAL,
	GOVERNING_TOKEN_OWNER,
	IS_RELINQUISHED,
	VOTER_WEIGHT,
	VOTE,
	NULL as VOTE_TYPE,
	NULL as VOTE_WEIGHT,
	SLOT,
	WRITE_VERSION
FROM VOTE_RECORDS_V2
WHERE (ADDRESS = ANY($1) OR $1 is null)
    AND (PROPOSAL = ANY($2) OR $2 is null)
    AND (GOVERNING_TOKEN_OWNER = ANY($3) OR $3 is null)
    AND (is_relinquished = $4 OR $4 is null);
 -- $1: addresses::text[]
 -- $2: proposals::text[]
 -- $3: governing_token_owners::text[]
 -- $4: is_relinquished::bool";

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
SELECT ADDRESS,
	ACCOUNT_TYPE,
	GOVERNANCE,
	GOVERNING_TOKEN_MINT,
	STATE,
	TOKEN_OWNER_RECORD,
	SIGNATORIES_COUNT,
	SIGNATORIES_SIGNED_OFF_COUNT,
	YES_VOTES_COUNT,
	NO_VOTES_COUNT,
	INSTRUCTIONS_EXECUTED_COUNT,
	INSTRUCTIONS_COUNT,
	INSTRUCTIONS_NEXT_INDEX,
	NULL AS VOTE_TYPE,
	NULL AS DENY_VOTE_WEIGHT,
	NULL AS VETO_VOTE_WEIGHT,
	NULL AS ABSTAIN_VOTE_WEIGHT,
	NULL AS START_VOTING_AT,
	DRAFT_AT,
	SIGNING_OFF_AT,
	VOTING_AT,
	NULL AS VOTING_AT_SLOT,
	VOTING_COMPLETED_AT,
	EXECUTING_AT,
	CLOSED_AT,
	EXECUTION_FLAGS,
	MAX_VOTE_WEIGHT,
	NULL AS MAX_VOTING_TIME,
	VOTE_THRESHOLD_TYPE,
	VOTE_THRESHOLD_PERCENTAGE,
	NAME,
	DESCRIPTION_LINK
FROM PROPOSALS_V1
WHERE (ADDRESS = ANY($1) or $1 is null) AND (governance = any($2) or $2 is null)
UNION ALL
SELECT ADDRESS,
	ACCOUNT_TYPE,
	GOVERNANCE,
	GOVERNING_TOKEN_MINT,
	STATE,
	TOKEN_OWNER_RECORD,
	SIGNATORIES_COUNT,
	SIGNATORIES_SIGNED_OFF_COUNT,
	NULL AS YES_VOTES_COUNT,
	NULL AS NO_VOTES_COUNT,
	NULL AS INSTRUCTIONS_EXECUTED_COUNT,
	NULL AS INSTRUCTIONS_COUNT,
	NULL AS INSTRUCTIONS_NEXT_INDEX,
	VOTE_TYPE,
	DENY_VOTE_WEIGHT,
	VETO_VOTE_WEIGHT,
	ABSTAIN_VOTE_WEIGHT,
	START_VOTING_AT,
	DRAFT_AT,
	SIGNING_OFF_AT,
	VOTING_AT,
	VOTING_AT_SLOT,
	VOTING_COMPLETED_AT,
	EXECUTING_AT,
	CLOSED_AT,
	EXECUTION_FLAGS,
	MAX_VOTE_WEIGHT,
	MAX_VOTING_TIME,
	VOTE_THRESHOLD_TYPE,
	VOTE_THRESHOLD_PERCENTAGE,
	NAME,
	DESCRIPTION_LINK
FROM PROPOSALS_V2
WHERE (ADDRESS = ANY($1) or $1 is null) AND (governance = any($2) or $2 is null);
 -- $1: addresses::text[]
 -- $2: governances::text[]";

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
