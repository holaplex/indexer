use super::prelude::*;

#[derive(Debug, Clone)]
pub struct NftAttribute {
    pub metadata_address: String,
    pub value: String,
    pub trait_type: String,
}

#[graphql_object(Context = AppContext)]
impl NftAttribute {
    pub fn metadata_address(&self) -> String {
        self.metadata_address.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub fn trait_type(&self) -> String {
        self.trait_type.clone()
    }
}

impl<'a> From<models::MetadataAttribute<'a>> for NftAttribute {
    fn from(
        models::MetadataAttribute {
            metadata_address,
            value,
            trait_type,
            ..
        }: models::MetadataAttribute,
    ) -> Self {
        Self {
            metadata_address: metadata_address.into_owned(),
            value: value.unwrap().into_owned(),
            trait_type: trait_type.unwrap().into_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NftCreator {
    pub address: String,
    pub metadata_address: String,
    pub share: i32,
    pub verified: bool,
}

#[graphql_object(Context = AppContext)]
impl NftCreator {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn metadata_address(&self) -> String {
        self.metadata_address.clone()
    }

    pub fn share(&self) -> i32 {
        self.share
    }

    pub fn verified(&self) -> bool {
        self.verified
    }
}

impl<'a> From<models::MetadataCreator<'a>> for NftCreator {
    fn from(
        models::MetadataCreator {
            creator_address,
            metadata_address,
            share,
            verified,
        }: models::MetadataCreator,
    ) -> Self {
        Self {
            address: creator_address.into_owned(),
            metadata_address: metadata_address.into_owned(),
            share,
            verified,
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct NftOwner {
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct Nft {
    pub address: String,
    pub name: String,
    pub seller_fee_basis_points: i32,
    pub mint_address: String,
    pub primary_sale_happened: bool,
    pub description: String,
    pub image: String,
}

impl From<models::Nft> for Nft {
    fn from(
        models::Nft {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            primary_sale_happened,
            description,
            image,
        }: models::Nft,
    ) -> Self {
        Self {
            address,
            name,
            seller_fee_basis_points,
            mint_address,
            primary_sale_happened,
            description: description.unwrap_or_else(String::new),
            image: image.unwrap_or_else(String::new),
        }
    }
}

#[graphql_object(Context = AppContext)]
impl Nft {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn mint_address(&self) -> String {
        self.mint_address.clone()
    }

    pub fn primary_sale_happened(&self) -> bool {
        self.primary_sale_happened
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn image(&self) -> String {
        self.image.clone()
    }

    pub async fn creators(&self, ctx: &AppContext) -> Vec<NftCreator> {
        ctx.nft_creator_loader.load(self.address.clone()).await
    }

    pub async fn attributes(&self, ctx: &AppContext) -> Vec<NftAttribute> {
        ctx.nft_attribute_loader.load(self.address.clone()).await
    }

    pub async fn owner(&self, ctx: &AppContext) -> Option<NftOwner> {
        ctx.nft_owner_loader.load(self.mint_address.clone()).await
    }
}
