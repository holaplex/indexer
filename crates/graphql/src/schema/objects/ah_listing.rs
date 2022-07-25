use indexer_core::uuid::Uuid;
use objects::{auction_house::AuctionHouse, nft::Nft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct AhListing {
    pub id: Uuid,
    pub trade_state: String,
    pub auction_house: PublicKey<AuctionHouse>,
    pub seller: PublicKey<Wallet>,
    pub metadata: PublicKey<Nft>,
    pub purchase_id: Option<Uuid>,
    pub price: U64,
    pub token_size: i32,
    pub trade_state_bump: i32,
    pub created_at: DateTime<Utc>,
    pub canceled_at: Option<DateTime<Utc>>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Auction house listing")]
impl AhListing {
    fn id(&self) -> &Uuid {
        &self.id
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

    fn purchase_id(&self) -> Option<Uuid> {
        self.purchase_id
    }

    fn price(&self) -> U64 {
        self.price
    }

    fn token_size(&self) -> i32 {
        self.token_size
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

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .auction_house_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a> TryFrom<models::Listing<'a>> for AhListing {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Listing {
            id,
            trade_state,
            auction_house,
            seller,
            metadata,
            purchase_id,
            price,
            token_size,
            trade_state_bump,
            created_at,
            canceled_at,
            ..
        }: models::Listing,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.unwrap_or_default(),
            trade_state: trade_state.into_owned(),
            auction_house: auction_house.into_owned().into(),
            seller: seller.into_owned().into(),
            metadata: metadata.into_owned().into(),
            purchase_id,
            price: price.try_into()?,
            token_size: token_size.try_into()?,
            trade_state_bump: trade_state_bump.into(),
            created_at: DateTime::from_utc(created_at, Utc),
            canceled_at: canceled_at.map(|c| DateTime::from_utc(c, Utc)),
        })
    }
}
