use scalars::Volume;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct MintStats {
    pub auction_house: String,
    pub mint: String,
    pub floor: Option<Volume>,
    pub average: Option<Volume>,
    pub volume_24hr: Option<Volume>,
}

impl<'a> TryFrom<models::MintStats<'a>> for MintStats {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::MintStats {
            auction_house,
            mint,
            floor,
            average,
            volume_24hr,
        }: models::MintStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            auction_house: auction_house.into_owned(),
            mint: mint.into_owned(),
            floor: floor.map(TryInto::try_into).transpose()?,
            average: average.map(TryInto::try_into).transpose()?,
            volume_24hr: volume_24hr.map(TryInto::try_into).transpose()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct MarketStats {
    pub nfts: Option<Volume>,
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
            nfts: nfts.map(TryInto::try_into).transpose()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct StoreCreatorStats {
    pub nfts: Option<Volume>,
}

impl<'a> TryFrom<models::StoreCreatorStats<'a>> for StoreCreatorStats {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::StoreCreatorStats {
            store_creator: _,
            nfts,
        }: models::StoreCreatorStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            nfts: nfts.map(TryInto::try_into).transpose()?,
        })
    }
}
