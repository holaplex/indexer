//! Query utilities for `graph_connections` table.

use diesel::OptionalExtension;

use crate::{
    db::{tables::twitter_handle_name_services, Connection},
    error::Result,
    prelude::*,
};

/// Return twitter handle linked to the provide wallet address
///
/// # Errors
/// This function fails if the underlying query fails to execute.
pub fn get(conn: &Connection, address: String) -> Result<Option<String>> {
    twitter_handle_name_services::table
        .filter(twitter_handle_name_services::wallet_address.eq(address))
        .select(twitter_handle_name_services::twitter_handle)
        .first::<String>(conn)
        .optional()
        .context("Failed to load twitter handle")
}
