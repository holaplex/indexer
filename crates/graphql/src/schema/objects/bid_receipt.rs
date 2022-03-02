use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "auction house bid receipt")]
pub struct BidReceipt {
    pub address: String,
    pub trade_state: String,
    pub buyer: String,
    pub metadata: String,
    pub auction_house: String,
    pub price: scalars::Lamports,
    pub trade_state_bump: i32,
    pub token_account: Option<String>,
    pub created_at: DateTime<Utc>,
    pub canceled_at: Option<DateTime<Utc>>,
}

impl<'a> TryFrom<models::BidReceipt<'a>> for BidReceipt {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::BidReceipt {
            address,
            trade_state,
            auction_house,
            bookkeeper: _,
            buyer,
            metadata,
            token_account,
            purchase_receipt: _,
            price,
            token_size: _,
            bump: _,
            trade_state_bump,
            created_at,
            canceled_at,
            ..
        }: models::BidReceipt,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            trade_state: trade_state.into_owned(),
            buyer: buyer.into_owned(),
            metadata: metadata.into_owned(),
            price: price.try_into()?,
            token_account: token_account.map(Cow::into_owned),
            auction_house: auction_house.into_owned(),
            trade_state_bump: trade_state_bump.into(),
            created_at: DateTime::from_utc(created_at, Utc),
            canceled_at: canceled_at.map(|c| DateTime::from_utc(c, Utc)),
        })
    }
}
