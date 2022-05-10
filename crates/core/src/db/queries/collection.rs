//! Queries to get nfts for collection and collection info
use diesel::{
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Array, Text},
};

use crate::{
    db::{models::Nft, Connection},
    error::prelude::*,
};

const COLLECTIONS_QUERY: &str = r"
    SELECT DISTINCT
        m.address,
        m.NAME,
        m.seller_fee_basis_points,
        m.mint_address,
        m.primary_sale_happened,
        m.uri, mj.description,
        mj.image, mj.category,
        mj.model
    FROM            metadatas m
    INNER JOIN      metadata_collection_keys c
    ON              m.mint_address = c.collection_address
    LEFT JOIN       metadata_jsons mj
    ON              m.address = mj.metadata_address
    WHERE           m.address = ANY($1);
    -- $1: address::text[]";

/// Load collection nft using collection address
/// collection nft address is the selection condition
///
/// # Errors
/// This function fails if the underlying SQL query returns an error
pub fn load_with_collection_address(
    conn: &Connection,
    addresses: impl ToSql<Array<Text>, Pg>,
) -> Result<Vec<Nft>> {
    diesel::sql_query(COLLECTIONS_QUERY)
        .bind(addresses)
        .load(conn)
        .context("Failed to load collection with collection address as parameter")
}
