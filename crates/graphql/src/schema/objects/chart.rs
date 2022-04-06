use indexer_core::db::queries::charts;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MintChart {
    pub auction_house: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct PricePoint {
    pub price: Option<i32>,
    pub date: String,
}

impl<'a> TryFrom<models::PricePoint> for PricePoint {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::PricePoint { price, date }: models::PricePoint,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            price: price.map(TryInto::try_into).transpose()?,
            date: date.to_string(),
        })
    }
}

#[graphql_object(Context = AppContext)]
impl MintChart {
    pub fn floor_price(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.db_pool.get()?;
        let rows =
            charts::floor_prices(&conn, &self.auction_house, self.start_date.naive_utc(), self.end_date.naive_utc())?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn average_price(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.db_pool.get()?;
        let rows =
            charts::average_prices(&conn, &self.auction_house, self.start_date.naive_utc(), self.end_date.naive_utc())?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}
