use objects::auction_house::AuctionHouse;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Marketplace {
    pub subdomain: String,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub auction_house_address: String,
}

impl<'a> From<models::StoreConfigJson<'a>> for Marketplace {
    fn from(
        models::StoreConfigJson {
            config_address: _,
            name,
            description,
            logo_url,
            banner_url,
            subdomain,
            owner_address: _,
            auction_house_address,
            ..
        }: models::StoreConfigJson,
    ) -> Self {
        Self {
            subdomain: subdomain.into_owned(),
            name: name.into_owned(),
            description: description.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.into_owned(),
            auction_house_address: auction_house_address.into_owned(),
        }
    }
}

#[juniper::graphql_object(Context = AppContext)]
impl Marketplace {
    pub fn subdomain(&self) -> String {
        self.subdomain.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn logo_url(&self) -> String {
        self.logo_url.clone()
    }

    pub fn banner_url(&self) -> String {
        self.banner_url.clone()
    }

    pub fn auction_house_address(&self) -> String {
        self.auction_house_address.clone()
    }

    pub async fn auction_house(&self, context: &AppContext) -> Vec<AuctionHouse> {
        context
            .auction_house_loader
            .load(self.auction_house_address.clone())
            .await
    }
}
