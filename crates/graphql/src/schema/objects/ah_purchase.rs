use indexer_core::uuid::Uuid;
use objects::{auction_house::AuctionHouse, nft::BaseNft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Purchase {
    pub id: Uuid,
    pub buyer: PublicKey<Wallet>,
    pub seller: PublicKey<Wallet>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub marketplace_program_address: String,
    pub metadata: PublicKey<BaseNft>,
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

    fn metadata(&self) -> &PublicKey<BaseNft> {
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

    fn marketplace_program_address(&self) -> &str {
        &self.marketplace_program_address
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<BaseNft>> {
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

impl<'a> TryFrom<models::Purchase<'a>> for Purchase {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Purchase {
            id,
            buyer,
            seller,
            auction_house,
            marketplace_program,
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
            marketplace_program_address: marketplace_program.into_owned(),
            created_at: DateTime::from_utc(created_at, Utc),
            token_size: token_size.try_into()?,
        })
    }
}
