use tables::storefronts;

use super::prelude::*;

pub type StorefrontColumns = (
    storefronts::owner_address,
    storefronts::subdomain,
    storefronts::title,
    storefronts::description,
    storefronts::favicon_url,
    storefronts::logo_url,
    storefronts::updated_at,
    storefronts::banner_url,
    storefronts::address,
);

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "A Metaplex storefront")]
pub struct Storefront {
    pub address: String,
    pub owner_address: String,
    pub subdomain: String,
    pub title: String,
    pub description: String,
    pub favicon_url: String,
    pub logo_url: String,
    pub banner_url: String,
}

impl<'a> From<models::Storefront<'a>> for Storefront {
    fn from(
        models::Storefront {
            address,
            owner_address,
            subdomain,
            title,
            description,
            favicon_url,
            logo_url,
            banner_url,
            ..
        }: models::Storefront,
    ) -> Self {
        Self {
            address: address.into_owned(),
            owner_address: owner_address.into_owned(),
            subdomain: subdomain.into_owned(),
            title: title.into_owned(),
            description: description.into_owned(),
            favicon_url: favicon_url.into_owned(),
            logo_url: logo_url.into_owned(),
            banner_url: banner_url.map_or_else(String::new, Cow::into_owned),
        }
    }
}
