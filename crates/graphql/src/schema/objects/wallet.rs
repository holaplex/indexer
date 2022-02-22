use objects::listing::Bid;
use tables::bids;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
}

#[graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        let db_conn = ctx.db_pool.get()?;

        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)
            .filter(bids::bidder_address.eq(self.address.clone()))
            .order_by(bids::last_bid_time.desc())
            .load(&db_conn)
            .context("Failed to load wallet bids")?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}
