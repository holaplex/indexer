//! Query utilities for collections.

use anyhow::Context;
use diesel::{pg::Pg, sql_types::{Integer, Text, Nullable, Array}, types::ToSql, RunQueryDsl};

use crate::{
    db::{custom_types::OrderDirection, models::Nft, Connection},
    error::Result,
};

const COLLECTIONS_BY_VOLUME_QUERY: &str = r"
select
    metadatas.address,
    metadatas.name,
    metadatas.seller_fee_basis_points,
    metadatas.update_authority_address,
    metadatas.mint_address,
    metadatas.primary_sale_happened,
    metadatas.uri,
    metadatas.slot,
    metadata_jsons.description,
    metadata_jsons.image,
    metadata_jsons.category,
    metadata_jsons.model
from metadata_jsons
inner join metadatas on (metadatas.address = metadata_jsons.metadata_address)
inner join (
    select metadata_collection_keys.collection_address as collection, sum(purchase_receipts.price) as volume
        from purchase_receipts
        inner join metadatas on (purchase_receipts.metadata = metadatas.address)
        inner join metadata_collection_keys on (metadatas.address = metadata_collection_keys.metadata_address)
        where ($1 IS NULL OR metadata_collection_keys.collection_address = ANY($1))
        group by metadata_collection_keys.collection_address
        order by volume {order_direction}
		limit $2
		offset $3
) a on (a.collection = metadatas.mint_address)
-- $2: limit::text[]
-- $2: limit::integer
-- $3: offset::integer";

/// Query collections ordered by volume
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn by_volume(
    conn: &Connection,
    addresses: impl ToSql<Nullable<Array<Text>>, Pg>,
    order_direction: OrderDirection,
    limit: impl ToSql<Integer, Pg>,
    offset: impl ToSql<Integer, Pg>,
) -> Result<Vec<Nft>> {
    let query =
        COLLECTIONS_BY_VOLUME_QUERY.replace("{order_direction}", &order_direction.to_string());
    diesel::sql_query(query)
        .bind(addresses)
        .bind(limit)
        .bind(offset)
        .load(conn)
        .context("Failed to load collections by volume")
}
