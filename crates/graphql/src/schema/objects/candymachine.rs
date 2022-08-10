use objects::wallet::Wallet;
use scalars::{PublicKey, U64};

use super::prelude::*;
use crate::schema::scalars::markers::TokenMint;

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachine {
    pub address: PublicKey<CandyMachine>,
    pub authority: PublicKey<Wallet>,
    pub wallet: PublicKey<Wallet>,
    pub token_mint: Option<PublicKey<TokenMint>>,
    pub items_redeemed: U64,

    pub uuid: String,
    pub price: U64,
    pub symbol: String,
    pub seller_fee_basis_points: i32,
    pub max_supply: U64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub go_live_date: Option<U64>,
    pub items_available: U64,
}

impl<'a, 'b> TryFrom<(models::CandyMachine<'a>, models::CandyMachineData<'b>)> for CandyMachine {
    type Error = std::num::TryFromIntError;

    fn try_from(
        (
            models::CandyMachine {
                address,
                authority,
                wallet,
                token_mint,
                items_redeemed,
            },
            models::CandyMachineData {
                uuid,
                price,
                symbol,
                seller_fee_basis_points,
                max_supply,
                is_mutable,
                retain_authority,
                go_live_date,
                items_available,
                ..
            },
        ): (models::CandyMachine, models::CandyMachineData),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into(),
            authority: authority.into(),
            wallet: wallet.into(),
            token_mint: token_mint.map(Into::into),
            items_redeemed: items_redeemed.try_into()?,
            uuid: uuid.into_owned(),
            price: price.try_into()?,
            symbol: symbol.into_owned(),
            seller_fee_basis_points: seller_fee_basis_points.try_into()?,
            max_supply: max_supply.try_into()?,
            is_mutable,
            retain_authority,
            go_live_date: go_live_date.map(U64::try_from).transpose()?,
            items_available: items_available.try_into()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineCreator {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub creator_address: PublicKey<Wallet>,
    pub verified: bool,
    pub share: i32,
}

impl<'a> TryFrom<models::CMCreator<'a>> for CandyMachineCreator {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CMCreator {
            candy_machine_address,
            creator_address,
            verified,
            share,
        }: models::CMCreator,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            candy_machine_address: candy_machine_address.into(),
            creator_address: creator_address.into(),
            verified,
            share: share.try_into()?,
        })
    }
}
