use std::collections::HashMap;

use indexer_core::db::models;
use itertools::Itertools;
use objects::attributes::{AttributeGroup, AttributeVariant};

use super::prelude::*;

/// groups metadata attributes into attribute groups
pub fn group(
    metadata_attributes: Vec<models::MetadataAttribute>,
) -> FieldResult<Vec<AttributeGroup>> {
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
                .sorted()
                .collect(),
        })
        .sorted()
        .collect::<Vec<_>>())
}
