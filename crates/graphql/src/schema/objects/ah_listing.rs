use objects::{auction_house::AuctionHouse, nft::Nft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Listing {
    pub trade_state: String,
    pub seller: PublicKey<Wallet>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub metadata: PublicKey<Nft>,
    pub price: U64,
    pub token_size: i32,
    pub created_at: DateTime<Utc>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Auction house listing")]
impl Listing {
    fn trade_state(&self) -> &str {
        &self.trade_state
    }

    fn seller(&self) -> &PublicKey<Wallet> {
        &self.seller
    }

    fn auction_house(&self) -> &PublicKey<AuctionHouse> {
        &self.auction_house
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

    fn token_size(&self) -> i32 {
        self.token_size
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::Listing<'a>> for Listing {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Listing {
            trade_state,
            auction_house,
            seller,
            metadata,

            price,
            token_size,

            created_at,
            ..
        }: models::Listing,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            trade_state: trade_state.into_owned(),
            auction_house: auction_house.into_owned().into(),
            seller: seller.into_owned().into(),
            metadata: metadata.into_owned().into(),
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
            token_size: token_size.try_into()?,
        })
    }
}
