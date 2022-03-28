use std::collections::HashMap;

use indexer_core::{db::queries::stats, prelude::*};
use itertools::Itertools;
use objects::{auction_house::AuctionHouse, stats::MintStats};
use tables::{attributes, metadata_creators};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
pub struct Creator {
    pub address: String,
}

#[derive(Debug, Clone, GraphQLObject, PartialEq, Eq, PartialOrd, Ord)]
struct AttributeVariant {
    name: String,
    count: i32,
}

#[derive(Debug, GraphQLObject, PartialEq, Eq, PartialOrd, Ord)]
struct AttributeGroup {
    name: String,
    variants: Vec<AttributeVariant>,
}
#[derive(Debug, Clone)]
struct CreatorCounts {
    creator: Creator,
}

impl CreatorCounts {
    #[must_use]
    pub fn new(creator: Creator) -> Self {
        Self { creator }
    }
}

#[graphql_object(Context = AppContext)]
impl CreatorCounts {
    fn creations(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;

        let count = metadata_creators::table
            .filter(metadata_creators::creator_address.eq(&self.creator.address))
            .filter(metadata_creators::verified.eq(true))
            .count()
            .get_result::<i64>(&conn)?;

        Ok(i64::try_into(count)?)
    }
}

#[graphql_object(Context = AppContext)]
impl Creator {
    fn address(&self) -> &str {
        &self.address
    }

    fn counts(&self) -> CreatorCounts {
        CreatorCounts::new(self.clone())
    }

    #[graphql(arguments(auction_houses(description = "Auction house public keys")))]
    pub async fn stats(
        &self,
        auction_houses: Vec<PublicKey<AuctionHouse>>,
        ctx: &AppContext,
    ) -> FieldResult<Vec<MintStats>> {
        let conn = ctx.db_pool.get()?;
        let rows = stats::collection(&conn, auction_houses, &self.address)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.db_pool.get()?;

        let metadata_attributes: Vec<models::MetadataAttribute> = attributes::table
            .inner_join(
                metadata_creators::table
                    .on(attributes::metadata_address.eq(metadata_creators::metadata_address)),
            )
            .filter(metadata_creators::creator_address.eq(&self.address))
            .filter(metadata_creators::verified.eq(true))
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
                    .sorted()
                    .collect(),
            })
            .sorted()
            .collect::<Vec<_>>())
    }
}
