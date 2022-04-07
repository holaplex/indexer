use objects::{listing::Bid, profile::TwitterProfile};
use tables::{bids, graph_connections};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: PublicKey<Wallet>,
    pub twitter_handle: Option<String>,
}

impl Wallet {
    pub fn new(address: PublicKey<Wallet>, twitter_handle: Option<String>) -> Self {
        Self {
            address,
            twitter_handle,
        }
    }
}

#[graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> &PublicKey<Wallet> {
        &self.address
    }

    pub fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        let db_conn = ctx.db_pool.get()?;
        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)      // How to avoid cloning?
            .filter(bids::bidder_address.eq::<String>(self.address.clone().into()))
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

    pub fn connection_counts(&self) -> FieldResult<ConnectionCounts> {
        Ok(ConnectionCounts {
            address: self.address.clone(),
        })
    }
}

pub struct ConnectionCounts {
    pub address: PublicKey<Wallet>,
}

#[graphql_object(Context = AppContext)]
impl ConnectionCounts {
    pub fn from_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.db_pool.get()?;
        let count = graph_connections::table
            .filter(graph_connections::from_account.eq::<String>(self.address.clone().into()))
            .count()
            .get_result::<i64>(&db_conn)
            .context("Failed to count from_connections")?;
        Ok(count.try_into()?)
    }
    pub fn to_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.db_pool.get()?;
        let count = graph_connections::table
            .filter(graph_connections::to_account.eq::<String>(self.address.clone().into()))
            .count()
            .get_result::<i64>(&db_conn)
            .context("Failed to count to_connections")?;
        Ok(count.try_into()?)
    }
}
