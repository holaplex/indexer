use objects::{listing::Bid, profile::TwitterProfile};
use tables::bids;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub twitter_handle: Option<String>,
}

impl Wallet {
    pub fn new(address: String, twitter_handle: Option<String>) -> Self {
        Self {
            address,
            twitter_handle,
        }
    }
}

#[graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        let db_conn = ctx.db_pool.get()?;

        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)
            .filter(bids::bidder_address.eq(&self.address))
            .order_by(bids::last_bid_time.desc())
            .load(&db_conn)
            .context("Failed to load wallet bids")?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = self.twitter_handle.clone();

        if twitter_handle.is_none() {
            return Ok(None);
        }

        let twitter_handle = twitter_handle.unwrap();

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }
}
