use std::marker::PhantomData;

use objects::{auction_house::AuctionHouse, stats::MarketStats, store_creator::StoreCreator};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
/// An Holaplex marketplace
pub struct Marketplace {
    pub config_address: String,
    pub subdomain: String,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub owner_address: String,
    pub auction_house_address: Option<PublicKey<AuctionHouse>>,
    pub store_address: Option<String>,
}

impl<'a> From<models::StoreConfigJson<'a>> for Marketplace {
    fn from(
        models::StoreConfigJson {
            config_address,
            name,
            description,
            logo_url,
            banner_url,
            subdomain,
            owner_address,
            auction_house_address,
            store_address,
        }: models::StoreConfigJson,
    ) -> Self {
        Self {
            config_address: config_address.into_owned(),
            subdomain: subdomain.into_owned(),
            name: name.into_owned(),
            description: description.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.into_owned(),
            owner_address: owner_address.into_owned(),
            auction_house_address: auction_house_address.map(|a| a.into()),
            store_address: store_address.map(Cow::into_owned),
        }
    }
}

#[graphql_object(Context = AppContext)]
impl Marketplace {
    pub fn config_address(&self) -> &str {
        &self.config_address
    }

    pub fn subdomain(&self) -> &str {
        &self.subdomain
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn logo_url(&self) -> &str {
        &self.logo_url
    }

    pub fn banner_url(&self) -> &str {
        &self.banner_url
    }

    pub fn owner_address(&self) -> &str {
        &self.owner_address
    }

    #[deprecated(note = "Use `auction_houses` instead")]
    pub fn auction_house_address(&self) -> Option<PublicKey<AuctionHouse>> {
        self.auction_house_address.clone()
    }

    pub fn store_address(&self) -> Option<&str> {
        self.store_address.as_deref()
    }

    #[deprecated(note = "Use `auction_houses` instead")]
    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        let ah = match self.auction_house_address {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };
        context
            .auction_house_loader
            .load(ah)
            .await
            .map_err(Into::into)
    }

    pub async fn auction_houses(&self, context: &AppContext) -> FieldResult<Vec<AuctionHouse>> {
        context
            .auction_houses_loader
            .load(self.config_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn creators(&self, context: &AppContext) -> FieldResult<Vec<StoreCreator>> {
        context
            .store_creator_loader
            .load(self.config_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn stats(&self, ctx: &AppContext) -> FieldResult<Option<MarketStats>> {
        ctx.market_stats_loader
            .load(self.config_address.clone().into())
            .await
            .map_err(Into::into)
    }
}
