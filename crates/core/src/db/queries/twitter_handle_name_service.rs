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

/// Return twitter handles linked to the provide wallet addresses
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get_multiple(
    conn: &Connection,
    addresses: Vec<String>,
) -> Result<Vec<TwitterHandle>> {

    twitter_handle_name_services::table
        .filter(twitter_handle_name_services::wallet_address.eq(any(addresses)))
        .select(twitter_handle_name_services::all_columns)
        .load(conn)
        .context("Failed to load twitter handle")
}
