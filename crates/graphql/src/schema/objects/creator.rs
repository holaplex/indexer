use std::collections::HashMap;

use indexer_core::{db::queries::stats, prelude::*};
use itertools::Itertools;
use objects::{auction_house::AuctionHouse, profile::TwitterProfile, stats::MintStats};
use scalars::PublicKey;
use tables::{attributes, metadata_creators};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Creator {
    pub address: String,
    pub twitter_handle: Option<String>,
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

        let count: i64 = metadata_creators::table
            .filter(metadata_creators::creator_address.eq(&self.creator.address))
            .filter(metadata_creators::verified.eq(true))
            .count()
            .get_result(&conn)?;

        Ok(count.try_into()?)
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

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }
}
