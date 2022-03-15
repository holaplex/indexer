//cdn selection
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use base64::display::Base64Display;
use indexer_core::assets::{AssetIdentifier, ImageSize};
use objects::{bid_receipt::BidReceipt, listing_receipt::ListingReceipt};
use reqwest::Url;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct NftAttribute {
    pub metadata_address: String,
    pub value: String,
    pub trait_type: String,
}

#[graphql_object(Context = AppContext)]
impl NftAttribute {
    pub fn metadata_address(&self) -> &str {
        &self.metadata_address
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn trait_type(&self) -> &str {
        &self.trait_type
    }
}

impl<'a> TryFrom<models::MetadataAttribute<'a>> for NftAttribute {
    type Error = Error;

    fn try_from(
        models::MetadataAttribute {
            metadata_address,
            value,
            trait_type,
            ..
        }: models::MetadataAttribute,
    ) -> Result<Self> {
        Ok(Self {
            metadata_address: metadata_address.into_owned(),
            value: value
                .ok_or_else(|| anyhow!("Missing attribute value"))?
                .into_owned(),
            trait_type: trait_type
                .ok_or_else(|| anyhow!("Missing attribute trait type"))?
                .into_owned(),
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct NftCreator {
    pub address: String,
    pub metadata_address: String,
    pub share: i32,
    pub verified: bool,
    pub position: Option<i32>,
}

impl<'a> From<models::MetadataCreator<'a>> for NftCreator {
    fn from(
        models::MetadataCreator {
            creator_address,
            metadata_address,
            share,
            verified,
            position,
        }: models::MetadataCreator,
    ) -> Self {
        Self {
            address: creator_address.into_owned(),
            metadata_address: metadata_address.into_owned(),
            share,
            verified,
            position,
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
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn seller_fee_basis_points(&self) -> i32 {
        self.seller_fee_basis_points
    }

    pub fn mint_address(&self) -> &str {
        &self.mint_address
    }

    pub fn primary_sale_happened(&self) -> bool {
        self.primary_sale_happened
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    #[graphql(arguments(width(description = "image width"),))]
    pub fn image(&self, width: Option<i32>, ctx: &AppContext) -> String {
        let width = ImageSize::from(width.unwrap_or(ImageSize::Medium as i32));

        let mut hasher = DefaultHasher::new();
        let assets_cdn = &ctx.asset_proxy_endpoint;
        let asset = AssetIdentifier::new(&Url::parse(&self.image).unwrap());

        if asset.arweave.is_some() && asset.ipfs.is_none() {
            let cid =
                Base64Display::with_config(&asset.arweave.unwrap().0, base64::URL_SAFE_NO_PAD)
                    .to_string();
            cid.hash(&mut hasher);
            let assets_cdn = assets_cdn.replace(
                "assets",
                &format!("assets{}", hasher.finish().rem_euclid(4)),
            );
            format!("{}/arweave/{}?width={}", assets_cdn, cid, width as i32)
        } else if asset.ipfs.is_some() && asset.arweave.is_none() {
            let cid = asset.ipfs.unwrap().to_string();
            cid.hash(&mut hasher);
            let assets_cdn = assets_cdn.replace(
                "assets",
                &format!("assets{}", hasher.finish().rem_euclid(4)),
            );
            format!("{}/ipfs/{}?width={}", assets_cdn, cid, width as i32)
        } else {
            String::from(&self.image)
        }
    }

    pub async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<NftCreator>> {
        ctx.nft_creators_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn attributes(&self, ctx: &AppContext) -> FieldResult<Vec<NftAttribute>> {
        ctx.nft_attributes_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn owner(&self, ctx: &AppContext) -> FieldResult<Option<NftOwner>> {
        ctx.nft_owner_loader
            .load(self.mint_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<ListingReceipt>> {
        ctx.listing_receipts_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn offers(&self, ctx: &AppContext) -> FieldResult<Vec<BidReceipt>> {
        ctx.bid_receipts_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }
}
