use objects::{auction_house::AuctionHouse, nft::Nft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct PurchaseReceipt {
    pub address: String,
    pub buyer: PublicKey<Wallet>,
    pub metadata: PublicKey<Nft>,
    pub seller: PublicKey<Wallet>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub price: U64,
    pub created_at: DateTime<Utc>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "A NFT bill of purchase")]
impl PurchaseReceipt {
    fn address(&self) -> &str {
        &self.address
    }

    fn buyer(&self) -> &PublicKey<Wallet> {
        &self.buyer
    }

    fn seller(&self) -> &PublicKey<Wallet> {
        &self.seller
    }

    fn metadata(&self) -> &PublicKey<Nft> {
        &self.metadata
    }

    fn price(&self) -> U64 {
        self.price
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .store_auction_houses_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::PurchaseReceipt<'a>> for PurchaseReceipt {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::PurchaseReceipt {
            address,
            buyer,
            seller,
            price,
            created_at,
            auction_house,
            metadata,
            ..
        }: models::PurchaseReceipt,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            auction_house: auction_house.into_owned().into(),
            buyer: buyer.into_owned().into(),
            metadata: metadata.into_owned().into(),
            seller: seller.into_owned().into(),
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
        })
    }
}
