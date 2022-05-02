use indexer_core::db::queries::charts;
use objects::{auction_house::AuctionHouse, creator::Creator};
use scalars::{PublicKey, U64};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct PriceChart {
    pub auction_houses: Vec<PublicKey<AuctionHouse>>,
    pub creators: Vec<PublicKey<Creator>>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct PricePoint {
    pub price: U64,
    pub date: DateTime<Utc>,
}

impl<'a> TryFrom<models::PricePoint> for PricePoint {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::PricePoint { price, date }: models::PricePoint,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            price: price.try_into()?,
            date: DateTime::from_utc(date, Utc),
        })
    }
}

#[graphql_object(Context = AppContext)]
impl PriceChart {
    pub fn listing_floor(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.shared.db.get()?;
        let rows = charts::floor_prices(
            &conn,
            &self.auction_houses,
            &self.creators,
            self.start_date.naive_utc(),
            self.end_date.naive_utc(),
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn sales_average(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.shared.db.get()?;
        let rows = charts::average_prices(
            &conn,
            &self.auction_houses,
            &self.creators,
            self.start_date.naive_utc(),
            self.end_date.naive_utc(),
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn total_volume(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.shared.db.get()?;
        let rows = charts::total_volume_prices(
            &conn,
            &self.auction_houses,
            &self.creators,
            self.start_date.naive_utc(),
            self.end_date.naive_utc(),
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}
