use indexer_core::db::queries::charts;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MintCharts {
    pub auction_house: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
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
impl MintCharts {
    pub fn floor_price(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.db_pool.get()?;
        let rows =
            charts::floor_prices(&conn, &self.auction_house, self.start_date, self.end_date)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn average_price(&self, ctx: &AppContext) -> FieldResult<Vec<PricePoint>> {
        let conn = ctx.db_pool.get()?;
        let rows =
            charts::average_prices(&conn, &self.auction_house, self.start_date, self.end_date)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}
