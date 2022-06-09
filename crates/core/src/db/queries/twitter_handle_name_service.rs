//! Query utilities for `graph_connections` table.

use diesel::{
    expression::{AsExpression, NonAggregate},
    pg::Pg,
    query_builder::{QueryFragment, QueryId},
    sql_types::{Array, Text},
    types::ToSql,
    AppearsOnTable, OptionalExtension,
};

use crate::{
    db::{models::TwitterHandle, tables::twitter_handle_name_services, Connection},
    error::Result,
    prelude::*,
};

/// Return twitter handle linked to the provide wallet address
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get<A: AsExpression<Text>>(conn: &Connection, address: A) -> Result<Option<String>>
where
    A::Expression: NonAggregate
        + QueryId
        + QueryFragment<Pg>
        + AppearsOnTable<twitter_handle_name_services::table>,
{
    twitter_handle_name_services::table
        .filter(twitter_handle_name_services::wallet_address.eq(address))
        .select(twitter_handle_name_services::twitter_handle)
        .first(conn)
        .optional()
        .context("Failed to load twitter handle")
}

const GET_MULTIPLE_HANDLES_QUERY: &str = r"
SELECT A.*
FROM twitter_handle_name_services A
INNER JOIN (
    SELECT wallet_address, MAX(write_version) as write_version
    FROM twitter_handle_name_services
    WHERE wallet_address = ANY($1)
    GROUP BY wallet_address
) B on A.wallet_address = B.wallet_address AND A.write_version = B.write_version
-- $1: addresses::text[]";

/// Return twitter handles linked to the provide wallet addresses
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get_multiple(
    conn: &Connection,
    addresses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<TwitterHandle>> {
    // Appears to not be possible to select a row by unique key by the 
    // highest value in another row using diesel ORM
    // See https://stackoverflow.com/a/61964064 (and diesel gitter)
    diesel::sql_query(GET_MULTIPLE_HANDLES_QUERY)
        .bind(addresses)
        .load(conn)
        .context("Failed to load featured listings")
}
