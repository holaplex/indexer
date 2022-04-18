use indexer_core::db::models;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct Activity {
    pub address: String,
    pub metadata: String,
    pub metadata_image: String,
    pub metadata_name: String,
    pub auction_house: String,
    pub price: scalars::Lamports,
    pub created_at: DateTime<Utc>,
    pub wallets: Vec<String>,
    pub activity_type: String,
}

impl TryFrom<models::Activity> for Activity {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::Activity {
            address,
            metadata,
            metadata_image,
            metadata_name,
            auction_house,
            price,
            created_at,
            wallets,
            activity_type,
        }: models::Activity,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address,
            metadata,
            metadata_image,
            metadata_name,
            auction_house,
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
            wallets,
            activity_type,
        })
    }
}
