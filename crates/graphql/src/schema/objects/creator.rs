use indexer_core::{db::queries::stats, prelude::*};
use objects::{
    attributes::AttributeGroup, auction_house::AuctionHouse, profile::TwitterProfile,
    stats::MintStats,
};
use scalars::PublicKey;
use services;
use tables::{attributes, metadata_creators};

use super::prelude::*;

#[derive(Debug, Clone)]
/// A creator associated with a marketplace
pub struct Creator {
    pub address: String,
    pub twitter_handle: Option<String>,
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
        let conn = context.shared.db.get()?;

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
        let conn = ctx.shared.db.get()?;
        let rows = stats::collection(&conn, auction_houses, &self.address)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(arguments(
        auction_houses(description = "List of auction houses"),
        start_date(description = "Start date for which we want to get the average price"),
        end_date(description = "End date for which we want to get the average price")
    ))]

    pub fn attribute_groups(&self, context: &AppContext) -> FieldResult<Vec<AttributeGroup>> {
        let conn = context.shared.db.get()?;

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

        services::attributes::group(metadata_attributes)
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
