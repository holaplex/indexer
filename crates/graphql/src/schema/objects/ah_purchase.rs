use indexer_core::uuid::Uuid;
use objects::{auction_house::AuctionHouse, nft::Nft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Purchase {
    pub id: Uuid,
    pub buyer: PublicKey<Wallet>,
    pub seller: PublicKey<Wallet>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub metadata: PublicKey<Nft>,
    pub token_size: i32,
    pub price: U64,
    pub created_at: DateTime<Utc>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Auction house purchase")]
impl Purchase {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn buyer(&self) -> &PublicKey<Wallet> {
        &self.buyer
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

impl<'a> TryFrom<models::Purchase<'a>> for Purchase {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Purchase {
            id,
            buyer,
            seller,
            auction_house,
            metadata,
            token_size,
            price,
            created_at,
            ..
        }: models::Purchase,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.unwrap_or_default(),
            buyer: buyer.into_owned().into(),
            seller: seller.into_owned().into(),
            metadata: metadata.into_owned().into(),
            price: price.try_into()?,
            auction_house: auction_house.into_owned().into(),
            created_at: DateTime::from_utc(created_at, Utc),
            token_size: token_size.try_into()?,
        })
    }
}
