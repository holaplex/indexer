use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct ListingReceipt {
    pub address: String,
    pub trade_state: String,
    pub seller: String,
    pub metadata: String,
    pub auction_house: String,
    pub price: scalars::Lamports,
    pub trade_state_bump: i32,
    pub created_at: DateTime<Utc>,
    pub canceled_at: Option<DateTime<Utc>>,
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
            bookkeeper: _,
            purchase_receipt: _,
            token_size: _,
            bump: _,
            ..
        }: models::ListingReceipt,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            trade_state: trade_state.into_owned(),
            seller: seller.into_owned(),
            metadata: metadata.into_owned(),
            auction_house: auction_house.into_owned(),
            price: price.try_into()?,
            trade_state_bump: trade_state_bump.into(),
            created_at: DateTime::from_utc(created_at, Utc),
            canceled_at: canceled_at.map(|c| DateTime::from_utc(c, Utc)),
        })
    }
}
