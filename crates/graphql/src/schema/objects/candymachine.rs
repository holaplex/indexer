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

#[derive(Debug, Clone)]
pub struct CandyMachine {
    pub address: String,
    pub authority: String,
    pub wallet: String,
    pub token_mint: Option<String>,
    pub items_redeemed: U64,

    pub uuid: String,
    pub price: U64,
    pub symbol: String,
    pub seller_fee_basis_points: u16,
    pub max_supply: U64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub go_live_date: Option<U64>,
    pub items_available: i64,
}

#[graphql_object(Context = AppContext)]
impl CandyMachine {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn price(&self) -> U64 {
        self.price
    }

    pub fn token_mint(&self) -> Option<&str> {
        self.token_mint.as_deref()
    }
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
        // TODO(will): Is there a one liner for this?
        let go_live_converted: Option<U64>;
        if let Some(value) = go_live_date {
            go_live_converted = Some(value.try_into()?);
        } else {
            go_live_converted = None;
        }

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
            go_live_date: go_live_converted,
            items_available: items_available.try_into()?,
        })
    }
}
