use indexer_core::db::models;

use super::prelude::*;
use crate::schema::scalars::U64;

#[derive(Debug, Clone)]
/// A decorator for a Metaplex Auction House
pub struct RewardCenter {
    pub address: String,
    pub token_mint: String,
    pub auction_house: String,
    pub bump: i32,
    pub slot: U64,
    pub write_version: U64,
}

impl<'a> TryFrom<models::RewardCenter<'a>> for RewardCenter {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::RewardCenter {
            address,
            token_mint,
            auction_house,
            bump,
            slot,
            write_version,
        }: models::RewardCenter,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            token_mint: token_mint.into_owned(),
            auction_house: auction_house.into_owned(),
            bump: bump.into(),
            slot: slot.try_into()?,
            write_version: write_version.try_into()?,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl RewardCenter {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn token_mint(&self) -> &str {
        &self.token_mint
    }

    pub fn auction_house(&self) -> &str {
        &self.auction_house
    }

    pub fn bump(&self) -> i32 {
        self.bump
    }

    pub fn slot(&self) -> U64 {
        self.slot
    }

    pub fn write_version(&self) -> U64 {
        self.write_version
    }
}
