use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Collection {
    pub address: String,
    pub name: String,
    pub mint_address: String,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl Collection {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mint_address(&self) -> &str {
        &self.mint_address
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }
}

impl From<models::Nft> for Collection {
    fn from(
        models::Nft {
            address,
            name,
            seller_fee_basis_points: _,
            mint_address,
            primary_sale_happened: _,
            uri: _,
            description,
            image,
            category: _,
            model: _,
        }: models::Nft,
    ) -> Self {
        Self {
            address,
            name,
            mint_address,
            description,
            image,
        }
    }
}
