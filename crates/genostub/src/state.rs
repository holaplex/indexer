use anchor_lang::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Element {
    Earth = 0,
    Fire,
    Water,
    Wood,
    Metal,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct RentalAgreement {
    pub alchemist: Option<Pubkey>,
    pub rental_period: u64,
    pub rent: u64,
    pub rent_token: Pubkey,
    pub rent_token_decimals: u8,
    pub last_rent_payment: u64,
    pub next_payment_due: u64,
    pub grace_period: u64,
    pub open_market: bool,
}

#[account]
#[derive(Default)]
pub struct HabitatData {
    pub habitat_mint: Pubkey,
    pub level: u8,
    pub element: u8,
    pub genesis: bool,
    pub renewal_timestamp: u64,
    pub expiry_timestamp: u64,
    pub next_day_timestamp: u64,
    pub crystals_refined: u8,
    pub harvester: [u8; 32],
    pub rental_agreement: Option<RentalAgreement>,
    pub ki_harvested: u64,
    pub seeds_spawned: bool,
    pub is_sub_habitat: bool,
    pub parent_habitat: Option<Pubkey>,
    pub sub_habitats: [Option<Pubkey>; 2],
    pub harvester_royalty_bips: u16,
    pub harvester_open_market: bool,
    pub total_ki_harvested: u64,
    pub total_crystals_refined: u64,
    pub terraforming_habitat: Option<Pubkey>,
    pub active: bool,
    pub durability: u16,
    pub habitats_terraformed: u32,
    pub sequence: u64,
    pub guild: Option<u16>,
    pub sub_habitat_cooldown_timestamp: u64,
    pub harvester_settings_cooldown_timestamp: u64,
}
