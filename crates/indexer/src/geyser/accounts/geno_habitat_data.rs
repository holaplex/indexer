use genostub::state::{HabitatData, RentalAgreement};
use indexer_core::{
    db::{
        insert_into, models,
        tables::{geno_habitat_datas, geno_rental_agreements},
    },
    util,
};

use super::Client;
use crate::prelude::*;

async fn process_rent(
    client: &Client,
    addr: String,
    rent: RentalAgreement,
    slot: i64,
    write_version: i64,
) -> Result<()> {
    let row = models::GenoRentalAgreement {
        habitat_address: Owned(addr),
        alchemist: rent.alchemist.map(|a| Owned(a.to_string())),
        rental_period: rent
            .rental_period
            .try_into()
            .context("Rental period was too big to store")?,
        rent: rent
            .rent
            .try_into()
            .context("Rent amount was too big to store")?,
        rent_token: Owned(rent.rent_token.to_string()),
        rent_token_decimals: rent.rent_token_decimals.into(),
        last_rent_payment: util::unix_timestamp_unsigned(rent.last_rent_payment)
            .context("Failed to convert last rent payment timestamp")?,
        next_payment_due: util::unix_timestamp_unsigned(rent.next_payment_due)
            .context("Failed to convert rent payment due timestamp")?,
        grace_period: rent
            .grace_period
            .try_into()
            .context("Rent grace period was too big to store")?,
        open_market: rent.open_market,
        slot,
        write_version,
    };

    client
        .db()
        .run(move |db| {
            insert_into(geno_rental_agreements::table)
                .values(&row)
                .on_conflict(geno_rental_agreements::habitat_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert Genopets habitat data")?;

    Ok(())
}

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    habitat: HabitatData,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let addr = key.to_string();
    let slot = i64::try_from(slot).context("Slot was too big to store")?;
    let write_version =
        i64::try_from(write_version).context("Write version was too big to store")?;

    let _: [_; 2] = habitat.sub_habitats; // Sanity check

    let row = models::GenoHabitatData {
        address: Owned(addr.clone()),
        habitat_mint: Owned(habitat.habitat_mint.to_string()),
        level: habitat.level.into(),
        element: habitat.element.into(),
        genesis: habitat.genesis,
        renewal_timestamp: util::unix_timestamp_unsigned(habitat.renewal_timestamp)
            .context("Failed to convert habitat renewal timestamp")?,
        expiry_timestamp: util::unix_timestamp_unsigned(habitat.expiry_timestamp)
            .context("Failed to convert habitat expiry timestamp")?,
        next_day_timestamp: util::unix_timestamp_unsigned(habitat.next_day_timestamp)
            .context("Failed to convert habitat next-day timestamp")?,
        crystals_refined: habitat.crystals_refined.into(),
        harvester_bytes: Owned(habitat.harvester.to_vec()),
        ki_harvested: habitat
            .ki_harvested
            .try_into()
            .context("Habitat ki harvested was too big to store")?,
        seeds_spawned: habitat.seeds_spawned,
        is_sub_habitat: habitat.is_sub_habitat,
        parent_habitat: habitat.parent_habitat.map(|p| Owned(p.to_string())),
        sub_habitat_0: habitat.sub_habitats[0].map(|s| Owned(s.to_string())),
        sub_habitat_1: habitat.sub_habitats[1].map(|s| Owned(s.to_string())),
        harvester_royalty_bips: habitat.harvester_royalty_bips.into(),
        harvester_open_market: habitat.harvester_open_market,
        total_ki_harvested: habitat
            .total_ki_harvested
            .try_into()
            .context("Habitat total ki harvested was too big to store")?,
        total_crystals_refined: habitat
            .total_crystals_refined
            .try_into()
            .context("Habitat total crystals refined was too big to store")?,
        terraforming_habitat: habitat.terraforming_habitat.map(|t| Owned(t.to_string())),
        active: habitat.active,
        durability: habitat.durability.into(),
        habitats_terraformed: habitat
            .habitats_terraformed
            .try_into()
            .context("Habitat habitats-terraformed count was too big to store")?,
        sequence: habitat
            .sequence
            .try_into()
            .context("Habitat sequence was too big to store")?,
        guild: habitat.guild.map(Into::into),
        sub_habitat_cooldown_timestamp: util::unix_timestamp_unsigned(
            habitat.sub_habitat_cooldown_timestamp,
        )
        .context("Failed to convert sub-habitat cooldown timestamp")?,
        harvester_settings_cooldown_timestamp: util::unix_timestamp_unsigned(
            habitat.harvester_settings_cooldown_timestamp,
        )
        .context("Failed to convert harvester settings cooldown timestamp")?,
        slot,
        write_version,
        harvester: Owned(
            std::str::from_utf8(&habitat.harvester)
                .context("Failed to convert harvester to UTF-8")?
                .trim_end_matches('\0')
                .to_owned(),
        ),
    };

    client
        .db()
        .run(move |db| {
            insert_into(geno_habitat_datas::table)
                .values(&row)
                .on_conflict(geno_habitat_datas::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert Genopets habitat data")?;

    if let Some(rent) = habitat.rental_agreement {
        process_rent(client, addr, rent, slot, write_version).await?;
    }

    Ok(())
}
