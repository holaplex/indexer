use scalars::Volume;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct MintStats {
    pub mint: String,
    pub floor: Volume,
    pub average: Volume,
    pub volume_24hr: Volume,
}

impl<'a> TryFrom<models::MintStats<'a>> for MintStats {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::MintStats {
            auction_house: _,
            mint,
            floor,
            average,
            volume_24hr,
        }: models::MintStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            mint: mint.into_owned(),
            floor: floor.try_into()?,
            average: average.try_into()?,
            volume_24hr: volume_24hr.try_into()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct MarketStats {
    pub nfts: Volume,
}

impl<'a> TryFrom<models::MarketStats<'a>> for MarketStats {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::MarketStats {
            store_config: _,
            nfts,
        }: models::MarketStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            nfts: nfts.try_into()?,
        })
    }
}
