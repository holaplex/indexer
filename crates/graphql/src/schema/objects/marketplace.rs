use objects::{
    auction_house::AuctionHouse, stats::MarketStats, store_creator::StoreCreator,
    storefront::Storefront,
};

use super::prelude::*;
use crate::schema::scalars::{markers::StoreConfig, PublicKey};

#[derive(Debug, Clone)]
/// An Holaplex marketplace
pub struct Marketplace {
    pub config_address: PublicKey<StoreConfig>,
    pub subdomain: String,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub owner_address: String,
    pub store_address: Option<PublicKey<Storefront>>,
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
            store_address,
        }: models::StoreConfigJson,
    ) -> Self {
        Self {
            config_address: config_address.into(),
            subdomain: subdomain.into_owned(),
            name: name.into_owned(),
            description: description.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.into_owned(),
            owner_address: owner_address.into_owned(),
            store_address: store_address.map(Into::into),
        }
    }
}

#[graphql_object(Context = AppContext)]
impl Marketplace {
    pub fn config_address(&self) -> &PublicKey<StoreConfig> {
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

    pub fn store_address(&self) -> &Option<PublicKey<Storefront>> {
        &self.store_address
    }

    pub async fn auction_houses(&self, context: &AppContext) -> FieldResult<Vec<AuctionHouse>> {
        context
            .auction_houses_loader
            .load(self.config_address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn creators(&self, context: &AppContext) -> FieldResult<Vec<StoreCreator>> {
        context
            .store_creator_loader
            .load(self.config_address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn stats(&self, ctx: &AppContext) -> FieldResult<Option<MarketStats>> {
        ctx.market_stats_loader
            .load(self.config_address.clone())
            .await
            .map_err(Into::into)
    }
}
