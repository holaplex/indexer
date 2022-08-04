use scalars::{PublicKey, U64};
use tables::{candy_machine_datas, candy_machines};

use super::prelude::*;

pub type CandyMachineColumns = (
    candy_machines::address,
    candy_machines::authority,
    candy_machines::wallet,
    candy_machines::token_mint,
    candy_machines::items_redeemed,
    candy_machine_datas::candy_machine_address,
    candy_machine_datas::uuid,
    candy_machine_datas::price,
    candy_machine_datas::symbol,
    candy_machine_datas::seller_fee_basis_points,
    candy_machine_datas::max_supply,
    candy_machine_datas::is_mutable,
    candy_machine_datas::retain_authority,
    candy_machine_datas::go_live_date,
    candy_machine_datas::items_available,
);

pub const CANDY_MACHINE_COLUMNS: CandyMachineColumns = (
    candy_machines::address,
    candy_machines::authority,
    candy_machines::wallet,
    candy_machines::token_mint,
    candy_machines::items_redeemed,
    candy_machine_datas::candy_machine_address,
    candy_machine_datas::uuid,
    candy_machine_datas::price,
    candy_machine_datas::symbol,
    candy_machine_datas::seller_fee_basis_points,
    candy_machine_datas::max_supply,
    candy_machine_datas::is_mutable,
    candy_machine_datas::retain_authority,
    candy_machine_datas::go_live_date,
    candy_machine_datas::items_available,
);

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachine {
    pub address: String,
    pub authority: String,
    pub wallet: String,
    pub token_mint: Option<String>,
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

impl TryFrom<models::CandyMachineJoined> for CandyMachine {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CandyMachineJoined {
            address,
            authority,
            wallet,
            token_mint,
            items_redeemed,
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
        }: models::CandyMachineJoined,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address,
            authority,
            wallet,
            token_mint,
            items_redeemed: items_redeemed.try_into()?,
            uuid,
            price: price.try_into()?,
            symbol,
            seller_fee_basis_points: seller_fee_basis_points.try_into()?,
            max_supply: max_supply.try_into()?,
            is_mutable,
            retain_authority,
            go_live_date: go_live_date.map(U64::try_from).transpose()?,
            items_available: items_available.try_into()?,
        })
    }
}
