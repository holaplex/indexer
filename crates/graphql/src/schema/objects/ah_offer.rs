use indexer_core::uuid::Uuid;
use objects::{auction_house::AuctionHouse, nft::BaseNft, wallet::Wallet};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Offer {
    pub id: Uuid,
    pub trade_state: String,
    pub buyer: PublicKey<Wallet>,
    pub metadata: PublicKey<BaseNft>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub marketplace_program_address: String,
    pub price: U64,
    pub purchase_id: Option<Uuid>,
    pub trade_state_bump: i32,
    pub token_account: Option<String>,
    pub created_at: DateTime<Utc>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub token_size: i32,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Auction house offer")]
impl Offer {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn trade_state(&self) -> &str {
        &self.trade_state
    }

    fn buyer(&self) -> &PublicKey<Wallet> {
        &self.buyer
    }

    fn metadata(&self) -> &PublicKey<BaseNft> {
        &self.metadata
    }

    fn price(&self) -> U64 {
        self.price
    }

    fn purchase_id(&self) -> Option<Uuid> {
        self.purchase_id
    }

    fn trade_state_bump(&self) -> i32 {
        self.trade_state_bump
    }

    fn token_account(&self) -> &Option<String> {
        &self.token_account
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn canceled_at(&self) -> Option<DateTime<Utc>> {
        self.canceled_at
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

impl<'a> TryFrom<models::Offer<'a>> for Offer {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::Offer {
            id,
            trade_state,
            auction_house,
            marketplace_program,
            buyer,
            metadata,
            token_account,
            price,
            purchase_id,
            token_size,
            trade_state_bump,
            created_at,
            canceled_at,
            ..
        }: models::Offer,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.unwrap_or_default(),
            trade_state: trade_state.into_owned(),
            buyer: buyer.into_owned().into(),
            metadata: metadata.into_owned().into(),
            price: price.try_into()?,
            purchase_id,
            token_account: token_account.map(Cow::into_owned),
            auction_house: auction_house.into_owned().into(),
            marketplace_program_address: marketplace_program.into_owned(),
            trade_state_bump: trade_state_bump.into(),
            created_at: DateTime::from_utc(created_at, Utc),
            canceled_at: canceled_at.map(|c| DateTime::from_utc(c, Utc)),
            token_size: token_size.try_into()?,
        })
    }
}
