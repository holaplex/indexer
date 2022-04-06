//! Query utilities for `graph_connections` table.

use diesel::{
    pg::Pg,
    serialize::ToSql,
    sql_query,
    sql_types::{Array, Int4, Text},
};

use crate::{
    db::{models::TwitterEnrichedGraphConnection, Connection},
    error::Result,
    prelude::*,
};

const CONNECTIONS_QUERY: &str = r"
SELECT gc.address AS connection_address, from_account, to_account, fth.twitter_handle AS from_twitter_handle, tth.twitter_handle AS to_twitter_handle
    FROM graph_connections gc
    LEFT JOIN twitter_handle_name_services fth ON gc.from_account = fth.wallet_address
    LEFT JOIN twitter_handle_name_services tth ON gc.to_account = tth.wallet_address
    WHERE ($1 = '{}' OR from_account = ANY($1)) AND ($2 = '{}' OR to_account = ANY($2))
    ORDER BY connection_address
    LIMIT $3 OFFSET $4;
 -- $1: from::text[]
 -- $2: to::text[]
 -- $3: limit::integer
 -- $4: limit::integer
 ";

/// Return connections based on from and to filters with limits and offset
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn list(
    conn: &Connection,
    from: impl ToSql<Array<Text>, Pg>,
    to: impl ToSql<Array<Text>, Pg>,
    limit: impl ToSql<Int4, Pg>,
    offset: impl ToSql<Int4, Pg>,
) -> Result<Vec<TwitterEnrichedGraphConnection>> {
    sql_query(CONNECTIONS_QUERY)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("failed to load twitter enriched graph connections")
}
