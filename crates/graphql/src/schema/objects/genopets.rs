use indexer_core::{base64, db::models};
use scalars::{PublicKey, I64};

use super::prelude::*;

pub enum Todo {} // TODO

#[derive(Debug, Clone, GraphQLObject)]
#[allow(clippy::struct_excessive_bools)]
pub struct GenoHabitat {
    pub address: PublicKey<GenoHabitat>,
    pub habitat_mint: PublicKey<Todo>,
    pub level: i32,
    pub element: i32,
    pub genesis: bool,
    pub renewal_timestamp: DateTime<Utc>,
    pub expiry_timestamp: DateTime<Utc>,
    pub next_day_timestamp: DateTime<Utc>,
    pub crystals_refined: i32,
    pub harvester: String,
    // pub rental_agreement: Option<GenoRentalAgreement>, // TODO
    pub ki_harvested: I64,
    pub seeds_spawned: bool,
    pub is_sub_habitat: bool,
    pub parent_habitat: Option<PublicKey<GenoHabitat>>,
    pub sub_habitats: Vec<PublicKey<GenoHabitat>>,
    pub harvester_royalty_bips: i32,
    pub harvester_open_market: bool,
    pub total_ki_harvested: I64,
    pub total_crystals_refined: I64,
    pub terraforming_habitat: Option<PublicKey<Todo>>,
    pub active: bool,
    pub durability: i32,
    pub habitats_terraformed: i32,
    pub sequence: I64,
    pub guild: Option<i32>,
    pub sub_habitat_cooldown_timestamp: DateTime<Utc>,
}

impl<'a> From<models::GenoHabitatData<'a>> for GenoHabitat {
    fn from(
        models::GenoHabitatData {
            address,
            habitat_mint,
            level,
            element,
            genesis,
            renewal_timestamp,
            expiry_timestamp,
            next_day_timestamp,
            crystals_refined,
            harvester,
            ki_harvested,
            seeds_spawned,
            is_sub_habitat,
            parent_habitat,
            sub_habitat_0,
            sub_habitat_1,
            harvester_royalty_bips,
            harvester_open_market,
            total_ki_harvested,
            total_crystals_refined,
            terraforming_habitat,
            active,
            durability,
            habitats_terraformed,
            sequence,
            guild,
            sub_habitat_cooldown_timestamp,
            slot: _,
            write_version: _,
        }: models::GenoHabitatData,
    ) -> Self {
        Self {
            address: address.into(),
            habitat_mint: habitat_mint.into(),
            level: level.into(),
            element: element.into(),
            genesis,
            renewal_timestamp: DateTime::from_utc(renewal_timestamp, Utc),
            expiry_timestamp: DateTime::from_utc(expiry_timestamp, Utc),
            next_day_timestamp: DateTime::from_utc(next_day_timestamp, Utc),
            crystals_refined: crystals_refined.into(),
            harvester: base64::encode_config(harvester, base64::STANDARD_NO_PAD),
            // rental_agreement: rental_agreement.map(Into::into),
            ki_harvested: ki_harvested.into(),
            seeds_spawned,
            is_sub_habitat,
            parent_habitat: parent_habitat.map(Into::into),
            sub_habitats: sub_habitat_0
                .into_iter()
                .chain(sub_habitat_1)
                .map(Into::into)
                .collect(),
            harvester_royalty_bips,
            harvester_open_market,
            total_ki_harvested: total_ki_harvested.into(),
            total_crystals_refined: total_crystals_refined.into(),
            terraforming_habitat: terraforming_habitat.map(Into::into),
            active,
            durability,
            habitats_terraformed,
            sequence: sequence.into(),
            guild,
            sub_habitat_cooldown_timestamp: DateTime::from_utc(sub_habitat_cooldown_timestamp, Utc),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct GenoRentalAgreement {
    pub habitat_address: PublicKey<GenoHabitat>,
    pub alchemist: Option<PublicKey<Todo>>,
    pub rental_period: I64,
    pub rent: I64,
    pub rent_token: PublicKey<Todo>,
    pub rent_token_decimals: i32,
    pub last_rent_payment: DateTime<Utc>,
    pub next_payment_due: DateTime<Utc>,
    pub grace_period: I64,
    pub open_market: bool,
}

impl<'a> From<models::GenoRentalAgreement<'a>> for GenoRentalAgreement {
    fn from(
        models::GenoRentalAgreement {
            habitat_address,
            alchemist,
            rental_period,
            rent,
            rent_token,
            rent_token_decimals,
            last_rent_payment,
            next_payment_due,
            grace_period,
            open_market,
            slot: _,
            write_version: _,
        }: models::GenoRentalAgreement,
    ) -> Self {
        Self {
            habitat_address: habitat_address.into(),
            alchemist: alchemist.map(Into::into),
            rental_period: rental_period.into(),
            rent: rent.into(),
            rent_token: rent_token.into(),
            rent_token_decimals: rent_token_decimals.into(),
            last_rent_payment: DateTime::from_utc(last_rent_payment, Utc),
            next_payment_due: DateTime::from_utc(next_payment_due, Utc),
            grace_period: grace_period.into(),
            open_market,
        }
    }
}
