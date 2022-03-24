use base64::display::Base64Display;
use indexer_core::assets::{AssetIdentifier, ImageSize};
use objects::{
    bid_receipt::BidReceipt, listing_receipt::ListingReceipt, purchase_receipt::PurchaseReceipt,
};
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

    #[graphql(arguments(width(description = r"Image width possible values are:
- 0 (Original size)
- 100 (Tiny)
- 400 (XSmall)
- 600 (Small)
- 800 (Medium)
- 1400 (Large)

Any other value will return the original image size.

If no value is provided, it will return XSmall")))]
    pub fn image(&self, width: Option<i32>, ctx: &AppContext) -> FieldResult<String> {
        fn get_asset_cdn(id: impl AsRef<[u8]>, shared: &SharedData) -> String {
            let rem = md5::compute(id).to_vec()[0].rem_euclid(shared.asset_proxy_count);
            let assets_cdn = &shared.asset_proxy_endpoint;

            if rem == 0 {
                assets_cdn.replace("[n]", "")
            } else {
                assets_cdn.replace("[n]", &rem.to_string())
            }
        }

        let width = ImageSize::from(width.unwrap_or(ImageSize::XSmall as i32));
        let id = AssetIdentifier::new(&Url::parse(&self.image).context("couldnt parse url")?);

        Ok(match (id.arweave, id.ipfs) {
            (Some(_), Some(_)) | (None, None) => self.image.clone(),
            (Some(txid), None) => {
                let txid = Base64Display::with_config(&txid.0, base64::URL_SAFE_NO_PAD).to_string();

                format!(
                    "{}arweave/{}?width={}",
                    get_asset_cdn(&txid, &ctx.shared),
                    txid,
                    width as i32
                )
            },
            (None, Some((cid, path))) => {
                let cid = cid.to_string();

                format!(
                    "{}ipfs/{}{}?width={}",
                    get_asset_cdn(&cid, &ctx.shared),
                    cid,
                    path,
                    width as i32
                )
            },
        })
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

    pub async fn purchases(&self, ctx: &AppContext) -> FieldResult<Vec<PurchaseReceipt>> {
        ctx.purchase_receipts_loader
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
