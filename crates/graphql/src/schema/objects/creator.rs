use std::collections::HashMap;

use indexer_core::prelude::*;
use tables::{attributes, metadata_creators};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Creator {
    pub address: String,
}

#[derive(Debug, Clone, GraphQLObject)]
struct AttributeVariant {
    name: String,
    count: i32,
}

#[derive(Debug, GraphQLObject)]
struct AttributeGroup {
    name: String,
    variants: Vec<AttributeVariant>,
}

#[graphql_object(Context = AppContext)]
impl Creator {
    fn address(&self) -> &str {
        &self.address
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.db_pool.get()?;

        let metadata_attributes: Vec<models::MetadataAttribute> = attributes::table
            .inner_join(
                metadata_creators::table
                    .on(attributes::metadata_address.eq(metadata_creators::metadata_address)),
            )
            .filter(metadata_creators::creator_address.eq(&self.address))
            .select(attributes::all_columns)
            .load(&conn)
            .context("Failed to load metadata attributes")?;

        Ok(metadata_attributes
            .into_iter()
            .try_fold(
                HashMap::new(),
                |mut groups,
                 models::MetadataAttribute {
                     trait_type, value, ..
                 }| {
                    *groups
                        .entry(
                            trait_type
                                .ok_or_else(|| anyhow!("Missing trait type from attribute"))?
                                .into_owned(),
                        )
                        .or_insert_with(HashMap::new)
                        .entry(value)
                        .or_insert(0) += 1;

                    Result::<_>::Ok(groups)
                },
            )?
            .into_iter()
            .map(|(name, vars)| AttributeGroup {
                name,
                variants: vars
                    .into_iter()
                    .map(|(name, count)| {
                        let name = name.map_or_else(String::new, Cow::into_owned);

                        AttributeVariant { name, count }
                    })
                    .collect(),
            })
            .collect::<Vec<_>>())
    }
}
