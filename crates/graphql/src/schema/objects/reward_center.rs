use indexer_core::db::models;

use super::prelude::*;
use objects::auction_house::AuctionHouse;
use crate::schema::{scalars::{U64, PublicKey, markers::TokenMint}, enums::PayoutOperation};

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
}
