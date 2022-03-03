use std::sync::Arc;

use diesel::{
    debug_query,
    pg::{expression::dsl::any, PgQueryBuilder},
    prelude::*,
};

use crate::{
    db::{
        schema,
        models::Nft,
        tables::{attributes, metadata_creators, metadata_jsons, metadatas, token_accounts},
        Connection, Pool,
    },
    error::prelude::*,
    prelude::debug,
};
#[derive(Debug)]
pub struct MetadataFilterAttributes {
    pub trait_type: String,
    pub values: Vec<String>,
}
type Q = diesel::query_builder::BoxedSelectStatement<'static,
    (diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Integer,
    diesel::sql_types::Text,
    diesel::sql_types::Text,
    diesel::sql_types::Bool,
    diesel::sql_types::Bool,
    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
    diesel::sql_types::Text),
    schema::metadatas::table,
    diesel::pg::Pg,
>;

fn build_attributes_query(attributes: Vec<MetadataFilterAttributes>, query: Q) -> Q {
    attributes
        .into_iter()
        .fold(query, |acc, MetadataFilterAttributes { trait_type, values }| {
            let sub = attributes::table
                .select(attributes::metadata_address)
                .filter(
                    attributes::trait_type
                        .eq(trait_type)
                        .and(attributes::value.eq(any(values))),
                );

            acc.filter(metadatas::address.eq(any(sub)))
        })
}

/// Handles queries for NFTs
///
/// # Errors
/// returns an error when the underlying queries throw an error
pub fn load_filtered(
    conn: &Connection,
    owners: Option<Vec<String>>,
    creators: Option<Vec<String>>,
    attributes: Option<Vec<MetadataFilterAttributes>>,
) -> Result<Vec<Nft>> {

    let mut query = metadatas::table.into_boxed();
    if let Some(attributes) = attributes {
        query = build_attributes_query(attributes, query);
    }

    let rows: Vec<Nft> = if let Some(creators) = creators {
        let creator_q = query
            .inner_join(
                metadata_creators::table
                    .on(metadatas::address.eq(metadata_creators::metadata_address)),
            )
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::seller_fee_basis_points,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .filter(metadata_creators::creator_address.eq(any(creators)))
            .order_by(metadatas::name.desc());

        let sql = debug_query::<diesel::pg::Pg, _>(&creator_q);
        let result = sql.to_string().replace("\"", "");
        debug!("THIS: {:?}", result);

        creator_q.load(conn).context("failed to load nft(s)")?
    } else if let Some(owners) = owners {
        // owners
        query
            .inner_join(
                token_accounts::table.on(metadatas::mint_address.eq(token_accounts::mint_address)),
            )
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(token_accounts::amount.eq(1))
            .filter(token_accounts::owner_address.eq(any(owners)))
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::seller_fee_basis_points,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .order_by(metadatas::name.desc())
            .load(conn)
            .context("failed to load nft(s)")?
    } else {
        unreachable!("something has gone horribly wrong on NFTs query");
    };

    Ok(rows.into_iter().map(Into::into).collect())
}
