use std::collections::HashMap;

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
    fn address(&self) -> String {
        self.address.clone()
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.db_pool.get()?;

        let metadatas: Vec<String> = metadata_creators::table
            .select(metadata_creators::metadata_address)
            .filter(metadata_creators::creator_address.eq(self.address.clone()))
            .load(&conn)
            .context("Failed to load metadata creators")?;

        let metadata_attributes: Vec<models::MetadataAttribute> = attributes::table
            .select(attributes::all_columns)
            .filter(attributes::metadata_address.eq(any(metadatas)))
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
                                .to_lowercase(),
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
