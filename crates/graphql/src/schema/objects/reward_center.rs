use indexer_core::{
    bigdecimal::ToPrimitive,
    db::{models, queries},
};
use objects::{auction_house::AuctionHouse, reward_payout::RewardPayout};

use super::prelude::*;
use crate::schema::{
    enums::PayoutOperation,
    scalars::{markers::TokenMint, PublicKey, U64},
};

#[derive(Debug, Clone)]
/// A decorator for a Metaplex Auction House
pub struct RewardCenter {
    pub address: PublicKey<Self>,
    pub token_mint: PublicKey<TokenMint>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub bump: i32,
    pub seller_reward_payout_basis_points: i32,
    pub mathematical_operand: PayoutOperation,
    pub payout_numeral: i32,
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
            seller_reward_payout_basis_points,
            mathematical_operand,
            payout_numeral,
            bump,
            slot,
            write_version,
        }: models::RewardCenter,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into(),
            token_mint: token_mint.into(),
            auction_house: auction_house.into(),
            bump: bump.into(),
            seller_reward_payout_basis_points: seller_reward_payout_basis_points.into(),
            mathematical_operand: mathematical_operand.into(),
            payout_numeral: payout_numeral.into(),
            slot: slot.try_into()?,
            write_version: write_version.try_into()?,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl RewardCenter {
    pub fn address(&self) -> &PublicKey<Self> {
        &self.address
    }

    pub fn token_mint(&self) -> &PublicKey<TokenMint> {
        &self.token_mint
    }

    pub fn auction_house(&self) -> &PublicKey<AuctionHouse> {
        &self.auction_house
    }

    pub fn bump(&self) -> i32 {
        self.bump
    }

    pub fn seller_reward_payout_basis_points(&self) -> i32 {
        self.seller_reward_payout_basis_points
    }

    pub fn mathematical_operand(&self) -> PayoutOperation {
        self.mathematical_operand
    }

    pub fn payout_numeral(&self) -> i32 {
        self.payout_numeral
    }

    pub fn slot(&self) -> U64 {
        self.slot
    }

    pub fn write_version(&self) -> U64 {
        self.write_version
    }

    pub fn payouts(
        &self,
        context: &AppContext,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<RewardPayout>> {
        let conn = context.shared.db.get()?;

        let payouts = queries::reward_centers::payouts(&conn, &self.address, limit, offset)?;

        payouts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn tokens_distributed(
        &self,
        context: &AppContext,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> FieldResult<Option<U64>> {
        let conn = context.shared.db.get()?;
        let start_date = start_date.unwrap_or_default();
        let end_date = end_date.unwrap_or(Utc::now());
        let result = queries::reward_centers::tokens_distributed(
            &conn,
            &self.address,
            start_date.naive_utc(),
            end_date.naive_utc(),
        )?;

        Ok(result
            .into_iter()
            .nth(0)
            .map(|models::TokensDistributed { tokens_distributed }| {
                tokens_distributed.to_u64().unwrap_or_default().into()
            }))
    }
}
