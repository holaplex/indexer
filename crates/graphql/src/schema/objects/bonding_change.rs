use scalars::{I64, U64};

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Bonding change enriched with reserve change and supply change")]
pub struct EnrichedBondingChange {
    pub address: String,
    pub slot: U64,
    pub insert_ts: NaiveDateTime,
    pub reserve_change: I64,
    pub supply_change: I64,
}

impl<'a> TryFrom<models::EnrichedBondingChange<'a>> for EnrichedBondingChange {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::EnrichedBondingChange {
            address,
            slot,
            insert_ts,
            reserve_change,
            supply_change,
            ..
        }: models::EnrichedBondingChange,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into_owned(),
            slot: slot.try_into()?,
            insert_ts,
            reserve_change: reserve_change.into(),
            supply_change: supply_change.into(),
        })
    }
}
