use objects::auction_house::AuctionHouse;
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MintStats {
    pub auction_house: PublicKey<AuctionHouse>,
    pub mint: String,
    pub floor: Option<U64>,
    pub average: Option<U64>,
    pub volume_24hr: Option<U64>,
    pub volume_total: Option<U64>,
}

#[graphql_object(Context = AppContext)]
#[graphql(description = "Mint Stats")]
impl MintStats {
    fn mint(&self) -> &str {
        &self.mint
    }

    fn floor(&self) -> Option<U64> {
        self.floor
    }

    fn average(&self) -> Option<U64> {
        self.average
    }

    fn volume_24hr(&self) -> Option<U64> {
        self.volume_24hr
    }

    fn volume_total(&self) -> Option<U64> {
        self.volume_total
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .store_auction_houses_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
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
            volume_total,
        }: models::MintStats,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            auction_house: auction_house.into(),
            mint: mint.into_owned(),
            floor: floor.map(TryInto::try_into).transpose()?,
            average: average.map(TryInto::try_into).transpose()?,
            volume_24hr: volume_24hr.map(TryInto::try_into).transpose()?,
            volume_total: volume_total.map(TryInto::try_into).transpose()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct MarketStats {
    pub nfts: Option<U64>,
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
