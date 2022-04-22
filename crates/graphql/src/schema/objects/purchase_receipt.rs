use scalars::U64;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "auction house bid receipt")]
pub struct PurchaseReceipt {
    pub address: String,
    pub buyer: String,
    pub seller: String,
    pub auction_house: String,
    pub price: U64,
    pub created_at: DateTime<Utc>,
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
            ..
        }: models::PurchaseReceipt,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            auction_house: auction_house.into_owned(),
            buyer: buyer.into_owned(),
            seller: seller.into_owned(),
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
        })
    }
}
