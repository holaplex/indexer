use base64::display::Base64Display;
use indexer_core::{
    assets::{AssetHint, AssetIdentifier, ImageSize},
    db::queries,
};
use objects::{
    auction_house::AuctionHouse, bid_receipt::BidReceipt, listing_receipt::ListingReceipt,
    profile::TwitterProfile, purchase_receipt::PurchaseReceipt,
};
use reqwest::Url;

use super::prelude::*;
use crate::schema::scalars::PublicKey;

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

#[derive(Debug, Clone)]
pub struct NftCreator {
    pub address: String,
    pub metadata_address: String,
    pub share: i32,
    pub verified: bool,
    pub position: Option<i32>,
    pub twitter_handle: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl NftCreator {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn metadata_address(&self) -> &str {
        &self.metadata_address
    }

    pub fn share(&self) -> i32 {
        self.share
    }

    pub fn verified(&self) -> bool {
        self.verified
    }

    pub fn position(&self) -> Option<i32> {
        self.position
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
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
}

impl<'a> From<(Option<String>, models::MetadataCreator<'a>)> for NftCreator {
    fn from(
        (
            twitter_handle,
            models::MetadataCreator {
                creator_address,
                metadata_address,
                share,
                verified,
                position,
            },
        ): (Option<String>, models::MetadataCreator),
    ) -> Self {
        Self {
            address: creator_address.into_owned(),
            metadata_address: metadata_address.into_owned(),
            share,
            verified,
            position,
            twitter_handle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NftOwner {
    pub address: String,
    pub associated_token_account_address: String,
    pub twitter_handle: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl NftOwner {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn associated_token_account_address(&self) -> &str {
        &self.associated_token_account_address
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
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
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct NftActivity {
    pub address: String,
    pub metadata: String,
    pub auction_house: String,
    pub price: scalars::Lamports,
    pub created_at: DateTime<Utc>,
    pub wallets: Vec<String>,
    pub activity_type: String,
}

impl TryFrom<models::NftActivity> for NftActivity {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::NftActivity {
            address,
            metadata,
            auction_house,
            price,
            created_at,
            wallets,
            activity_type,
        }: models::NftActivity,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address,
            metadata,
            auction_house,
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
            wallets,
            activity_type,
        })
    }
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
        fn format_cdn_url<'a>(
            shared: &SharedData,
            id: &AssetIdentifier,
            hint: AssetHint,
            path: impl IntoIterator<Item = &'a str>,
            query: impl IntoIterator<Item = (&'a str, &'a str)>,
        ) -> Url {
            let rem = md5::compute(
                id.fingerprint(Some(hint))
                    .unwrap_or_else(|| unreachable!())
                    .as_ref(),
            )[0]
            .rem_euclid(shared.asset_proxy_count);
            let assets_cdn = &shared.asset_proxy_endpoint;

            let mut url = Url::parse(&assets_cdn.replace(
                "[n]",
                &if rem == 0 {
                    String::new()
                } else {
                    rem.to_string()
                },
            ))
            .unwrap_or_else(|_| unreachable!());

            url.path_segments_mut()
                .unwrap_or_else(|_| unreachable!())
                .extend(path);
            url.query_pairs_mut().extend_pairs(query);

            url
        }

        let width = ImageSize::from(width.unwrap_or(ImageSize::XSmall as i32));
        let width_str = (width as i32).to_string();
        let id =
            AssetIdentifier::new(&Url::parse(&self.image).context("Couldn't parse asset URL")?);

        Ok(match (id.arweave, &id.ipfs) {
            (Some(_), Some(_)) | (None, None) => self.image.clone(),
            (Some(txid), None) => {
                let txid = Base64Display::with_config(&txid.0, base64::URL_SAFE_NO_PAD).to_string();

                format_cdn_url(
                    &ctx.shared,
                    &id,
                    AssetHint::Arweave,
                    ["arweave", &txid],
                    Some(("width", &*width_str)),
                )
                .to_string()
            },
            (None, Some((cid, path))) => {
                let cid = cid.to_string();

                format_cdn_url(
                    &ctx.shared,
                    &id,
                    AssetHint::Ipfs,
                    ["ipfs", &cid],
                    Some(("width", &*width_str))
                        .into_iter()
                        .chain(if path.is_empty() {
                            None
                        } else {
                            Some(("path", &**path))
                        }),
                )
                .to_string()
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

    pub async fn activities(&self, ctx: &AppContext) -> FieldResult<Vec<NftActivity>> {
        ctx.nft_activities_loader
            .load(self.address.clone().into())
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

#[derive(Debug, Clone)]
pub struct NftCount {
    creators: Vec<PublicKey<NftCreator>>,
}

impl NftCount {
    #[must_use]
    pub fn new(creators: Vec<PublicKey<NftCreator>>) -> Self {
        Self { creators }
    }
}

#[graphql_object(Context = AppContext)]
impl NftCount {
    fn total(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;
        let creators: Vec<String> = self.creators.clone().into_iter().map(Into::into).collect();
        let count = queries::nft_count::total(&conn, creators)?;
        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "a list of auction house public keys")))]
    fn listed(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.db_pool.get()?;
        let creators: Vec<String> = self
            .creators
            .clone()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let auction_houses = auction_houses.map(|a| a.into_iter().map(Into::into).collect());
        let count = queries::nft_count::listed(&conn, creators, auction_houses)?;
        Ok(count.try_into()?)
    }
}
