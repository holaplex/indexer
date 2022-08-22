//! Query utilities for NFT activity.

use anyhow::Context;
use diesel::{
    pg::Pg,
    sql_types::{Array, Bool, Nullable, Text},
    types::ToSql,
    RunQueryDsl,
};

use crate::{
    db::{models::VoteRecord, Connection},
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
WHERE ADDRESS = ANY($1)
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
