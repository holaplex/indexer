use objects::{
    auction_house::AuctionHouse, nft::Nft, purchase_receipt::PurchaseReceipt, wallet::Wallet,
};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct ListingReceipt {
    pub address: String,
    pub trade_state: String,
    pub seller: PublicKey<Wallet>,
    pub metadata: PublicKey<Nft>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub price: U64,
    pub trade_state_bump: i32,
    pub created_at: DateTime<Utc>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub bookkeeper: PublicKey<Wallet>,
    pub purchase_receipt: Option<PublicKey<PurchaseReceipt>>,
    pub token_size: i32,
    pub bump: i32,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "An NFT listing receipt")]
impl ListingReceipt {
    fn address(&self) -> &str {
        &self.address
    }

    fn trade_state(&self) -> &str {
        &self.trade_state
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

    fn trade_state_bump(&self) -> i32 {
        self.trade_state_bump
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn canceled_at(&self) -> Option<DateTime<Utc>> {
        self.canceled_at
    }

    fn bookkeeper(&self) -> &PublicKey<Wallet> {
        &self.bookkeeper
    }

    fn purchase_receipt(&self) -> &Option<PublicKey<PurchaseReceipt>> {
        &self.purchase_receipt
    }

    fn token_size(&self) -> i32 {
        self.token_size
    }

    fn bump(&self) -> i32 {
        self.bump
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

impl<'a> TryFrom<models::ListingReceipt<'a>> for ListingReceipt {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::ListingReceipt {
            address,
            trade_state,
            auction_house,
            seller,
            price,
            metadata,
            trade_state_bump,
            created_at,
            canceled_at,
            bookkeeper,
            purchase_receipt,
            token_size,
            bump,
            ..
        }: models::ListingReceipt,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            trade_state: trade_state.into_owned(),
            seller: seller.into_owned().into(),
            metadata: metadata.into_owned().into(),
            auction_house: auction_house.into_owned().into(),
            price: price.try_into()?,
            trade_state_bump: trade_state_bump.into(),
            created_at: DateTime::from_utc(created_at, Utc),
            canceled_at: canceled_at.map(|c| DateTime::from_utc(c, Utc)),
            bookkeeper: bookkeeper.into_owned().into(),
            purchase_receipt: purchase_receipt.map(|pr| pr.into_owned().into()),
            token_size: token_size.try_into()?,
            bump: bump.into(),
        })
    }
}
