use objects::{auction_house::AuctionHouse, store_creator::StoreCreator};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Marketplace {
    pub config_address: String,
    pub subdomain: String,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub owner_address: String,
    pub store_address: String,
    pub auction_house_address: String,
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
            auction_house_address,
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
            store_address: store_address.into_owned(),
            auction_house_address: auction_house_address.into_owned(),
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
    pub fn store_address(&self) -> &str {
        &self.store_address
    }
    pub fn auction_house_address(&self) -> &str {
        &self.auction_house_address
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .auction_house_loader
            .load(self.auction_house_address.clone().into())
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
}
