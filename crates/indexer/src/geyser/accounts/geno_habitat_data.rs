use genostub::state::{HabitatData, RentalAgreement};
use indexer_core::{
    db::{
        delete, insert_into, models, select,
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
        habitat_address: Owned(addr.clone()),
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
            let geno_rental = select(exists(
                geno_rental_agreements::table.filter(
                    geno_rental_agreements::habitat_address
                        .eq(addr.clone())
                        .and(geno_rental_agreements::slot.lt(slot)),
                ),
            ))
            .get_result::<bool>(db);

            if geno_rental == Ok(true) {
                delete(
                    geno_rental_agreements::table
                        .filter(geno_rental_agreements::habitat_address.eq(addr)),
                )
                .execute(db)?;
            }

            insert_into(geno_rental_agreements::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(db)
        })
        .await
        .context("Failed to insert Genopets habitat data")?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
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
        daily_ki_harvesting_cap: 0.into(),
        ki_available_to_harvest: None,
        has_max_ki: None,
    };

    client
        .db()
        .run(move |db| {
            let geno_exists = select(exists(
                geno_habitat_datas::table.filter(
                    geno_habitat_datas::address
                        .eq(key.to_string())
                        .and(geno_habitat_datas::slot.lt(slot)),
                ),
            ))
            .get_result::<bool>(db);

            if geno_exists == Ok(true) {
                delete(
                    geno_habitat_datas::table
                        .filter(geno_habitat_datas::address.eq(key.to_string())),
                )
                .execute(db)?;
            }

            insert_into(geno_habitat_datas::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(db)
        })
        .await
        .context("Failed to insert Genopets habitat data")?;

    if let Some(rent) = habitat.rental_agreement {
        process_rent(client, addr, rent, slot, write_version).await?;
    }

    client
        .search()
        .upsert_geno_habitat(false, habitat.habitat_mint)
        .await?;

    Ok(())
}
