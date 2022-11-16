use std::collections::HashMap;

use indexer_core::db::models;
use itertools::Itertools;
use objects::attributes::{AttributeGroup, AttributeVariant};

use super::prelude::*;

/// groups metadata attributes into attribute groups
pub fn group(metadata_attributes: Vec<models::AttributeGroup>) -> FieldResult<Vec<AttributeGroup>> {
    Ok(metadata_attributes
        .into_iter()
        .try_fold(
            HashMap::new(),
            |mut groups,
             models::AttributeGroup {
                 trait_type,
                 value,
                 count,
                 ..
             }| {
                groups
                    .entry(trait_type)
                    .or_insert_with(HashMap::new)
                    .entry(value)
                    .or_insert(count);

                Result::<_>::Ok(groups)
            },
        )?
        .into_iter()
        .map(|(name, vars)| AttributeGroup {
            name: name.to_string(),
            variants: vars
                .into_iter()
                .map(|(name, count)| AttributeVariant {
                    name: name.to_string(),
                    count: count.try_into().unwrap_or_default(),
                })
                .sorted()
                .collect(),
        })
        .sorted()
        .collect::<Vec<_>>())
}
