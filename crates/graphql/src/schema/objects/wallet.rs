use indexer_core::db::queries;
use objects::{
    auction_house::AuctionHouse, listing::Bid, nft::NftCreator, profile::TwitterProfile,
};
use scalars::PublicKey;
use tables::{bids, graph_connections};

use super::prelude::*;

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

#[derive(Debug, Clone)]
pub struct WalletNftCount {
    wallet: PublicKey<Wallet>,
    creators: Option<Vec<PublicKey<NftCreator>>>,
}

impl WalletNftCount {
    #[must_use]
    pub fn new(wallet: PublicKey<Wallet>, creators: Option<Vec<PublicKey<NftCreator>>>) -> Self {
        Self { wallet, creators }
    }
}

#[graphql_object(Context = AppContext)]
impl WalletNftCount {
    fn owned(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;
        let creators: Option<Vec<String>> = self
            .clone()
            .creators
            .map(|c| c.into_iter().map(Into::into).collect::<Vec<_>>());
        let count = queries::nft_count::owned(&conn, self.wallet.clone().into(), creators)?;
        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "auction houses to scope wallet counts")))]
    fn offered(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;
        let creators: Option<Vec<String>> = self
            .creators
            .clone()
            .map(|c| c.into_iter().map(Into::into).collect());
        let auction_houses = auction_houses.map(|a| a.into_iter().map(Into::into).collect());
        let count = queries::nft_count::offered(
            &conn,
            self.wallet.clone().into(),
            creators,
            auction_houses,
        )?;
        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "auction houses to scope wallet counts")))]
    fn listed(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;
        let creators: Option<Vec<String>> = self
            .creators
            .clone()
            .map(|c| c.into_iter().map(Into::into).collect());
        let auction_houses = auction_houses.map(|a| a.into_iter().map(Into::into).collect());
        let count = queries::nft_count::wallet_listed(
            &conn,
            self.wallet.clone().into(),
            creators,
            auction_houses,
        )?;
        Ok(count.try_into()?)
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
            .select(bids::all_columns)
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
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

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

    #[graphql(arguments(creators(description = "a list of auction house public keys")))]
    pub fn nft_counts(
        &self,
        _ctx: &AppContext,
        creators: Option<Vec<PublicKey<NftCreator>>>,
    ) -> WalletNftCount {
        WalletNftCount::new(self.address.clone(), creators)
    }
}

pub struct ConnectionCounts {
    pub address: PublicKey<Wallet>,
}

#[graphql_object(Context = AppContext)]
impl ConnectionCounts {
    pub fn from_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.db_pool.get()?;

        let count: i64 = graph_connections::table
            .filter(graph_connections::from_account.eq(&self.address))
            .count()
            .get_result(&db_conn)
            .context("Failed to count from_connections")?;

        Ok(count.try_into()?)
    }

    pub fn to_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.db_pool.get()?;

        let count: i64 = graph_connections::table
            .filter(graph_connections::to_account.eq(&self.address))
            .count()
            .get_result(&db_conn)
            .context("Failed to count to_connections")?;

        Ok(count.try_into()?)
    }
}
