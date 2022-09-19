//! Query utilities for Genopets

use std::ops::Bound;

use chrono::{naive::NaiveDateTime, prelude::*};
use diesel::{
    dsl::any,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{Nullable, Text},
};

use super::handle_range;
use crate::{
    db::{
        models::GenoHabitatData,
        tables::{current_metadata_owners, geno_habitat_datas, geno_rental_agreements},
        Connection,
    },
    error::prelude::*,
};

/// A habitat field by which to sort query results
#[derive(Debug, Clone, Copy)]
pub enum HabitatSortField {
    /// Sorty by `address`
    Address,
    /// Sort by `level`
    Level,
    /// Sort by habitat lifespan (`expiry_timestamp`)
    Lifespan,
    /// Sort by `ki_harvested`
    KiHarvested,
    /// Sort by `crystals_refined`
    CrystalsRefined,
    /// Sort by `total_ki_harvested`
    TotalKiHarvested,
    /// Sort by `ki_available_to_harvest`
    KiAvailableToHarvest,
}

/// Input parameters for the [`list_habitats`] query.
#[derive(Debug)]
pub struct ListHabitatOptions<M, W, H> {
    /// Select habitats by their NFT mint addresses
    pub mints: Option<Vec<M>>,
    /// Select only habitats owned by any of the given public keys
    pub owners: Option<Vec<W>>,
    /// Select only habitats rented by any of the given public keys
    pub renters: Option<Vec<W>>,
    /// Select only habitats with harvesters matching any of the given strings
    pub harvesters: Option<Vec<H>>,
    /// Select only habitats whose genesis flag matches the given value
    pub genesis: Option<bool>,
    /// Select only habitats whose element matches any of the given elements
    pub elements: Option<Vec<i16>>,
    /// Select only habitats whose level falls within the given range
    pub levels: (Bound<i16>, Bound<i16>),
    /// Select only habitats whose sequence falls within the given range
    pub sequences: (Bound<i64>, Bound<i64>),
    /// Select only habitats whose guild matches any of the given guilds
    pub guilds: Option<Vec<i32>>,
    /// Select only habitats whose durability falls within the given range
    pub durabilities: (Bound<i32>, Bound<i32>),
    /// Select only habitats whose expiry timestamps fall within the given range
    pub expiries: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
    /// Select only habitats whose harvester open-market flag matches the given
    /// value
    pub harvester_open_market: Option<bool>,
    /// Select only habitats whose rental agreement's open-market flag matches
    /// the given value
    pub rental_open_market: Option<bool>,
    /// Select only habitats having harvester/no harvester
    pub has_harvester: Option<bool>,
    /// Select only habitats having alchemist/no alchemist
    pub has_alchemist: Option<bool>,
    /// Select only habitats which have max ki i.e dailykicaplimit == ki_harvested
    pub has_max_ki: Option<bool>,
    /// Select activated habitats only i.e expiry_timestamp != 0
    pub is_activated: Option<bool>,
    /// Field to sort results on
    pub sort_field: HabitatSortField,
    /// True if rows should be sorted in descending order, false if they should
    /// be sorted ascending
    pub sort_desc: bool,
    /// Limit the number of returned rows
    pub limit: i64,
    /// Skip the first `n` resulting rows
    pub offset: i64,
}

/// Tuple of `(habitats, total_count_hint)`
pub type HabitatList = (Vec<GenoHabitatData<'static>>, Option<i64>);

/// List the Genopets `HabitatData` accounts matching the given query parameters
///
/// # Errors
/// This function fails if the underlying query returns an error.
#[allow(clippy::too_many_lines)] // splitting this function would require naming eldritch types
pub fn list_habitats<
    M: ToSql<Text, Pg>,
    W: ToSql<Text, Pg> + ToSql<Nullable<Text>, Pg>,
    H: ToSql<Text, Pg>,
>(
    conn: &Connection,
    opts: ListHabitatOptions<M, W, H>,
) -> Result<HabitatList> {
    let ListHabitatOptions {
        mints,
        owners,
        renters,
        harvesters,
        genesis,
        elements,
        levels,
        sequences,
        guilds,
        durabilities,
        expiries,
        harvester_open_market,
        rental_open_market,
        has_harvester,
        has_alchemist,
        has_max_ki,
        is_activated,
        sort_field,
        sort_desc,
        limit,
        offset,
    } = opts;

    let build_query = |sort| {
        let mut query = geno_habitat_datas::table.into_boxed();
        let mut count = true;

        if let Some(ref mints) = mints {
            query = query.filter(geno_habitat_datas::habitat_mint.eq(any(mints)));
        }

        if let Some(ref owners) = owners {
            query = query.filter(
                geno_habitat_datas::habitat_mint.eq(any(current_metadata_owners::table
                    .filter(current_metadata_owners::owner_address.eq(any(owners)))
                    .select(current_metadata_owners::mint_address))),
            );

            count = false;
        }

        if let Some(ref renters) = renters {
            query = query.filter(
                geno_habitat_datas::address.eq(any(geno_rental_agreements::table
                    .filter(geno_rental_agreements::alchemist.eq(any(renters)))
                    .select(geno_rental_agreements::habitat_address))),
            );

            count = false;
        }

        if let Some(ref harvesters) = harvesters {
            query = query.filter(geno_habitat_datas::harvester.eq(any(harvesters)));
        }

        if let Some(genesis) = genesis {
            query = query.filter(geno_habitat_datas::genesis.eq(genesis));
        }

        if has_harvester == Some(true) {
            query = query.filter(geno_habitat_datas::harvester.ne(""));
        }

        if has_harvester == Some(false) {
            query = query.filter(geno_habitat_datas::harvester.eq(""));
        }

        if is_activated == Some(false) {
            query = query.filter(geno_habitat_datas::expiry_timestamp.eq(NaiveDateTime::MIN));
        }

        if let Some(has_max_ki) = has_max_ki {
            query = query.filter(geno_habitat_datas::has_max_ki.eq(has_max_ki));
        }

        if let Some(ref elements) = elements {
            query = query.filter(geno_habitat_datas::element.eq(any(elements)));
        }

        query = handle_range(query, geno_habitat_datas::level, levels);
        query = handle_range(query, geno_habitat_datas::sequence, sequences);

        if let Some(ref guilds) = guilds {
            query = query.filter(geno_habitat_datas::guild.eq(any(guilds)));
        }

        query = handle_range(query, geno_habitat_datas::durability, durabilities);

        {
            let (min, max) = expiries;
            let min = min.map(|m| m.naive_utc());
            let max = max.map(|m| m.naive_utc());

            query = handle_range(query, geno_habitat_datas::expiry_timestamp, (min, max));
        }

        if let Some(harvester_open_market) = harvester_open_market {
            query =
                query.filter(geno_habitat_datas::harvester_open_market.eq(harvester_open_market));
        }

        if has_alchemist == Some(true) {
            query = query.filter(
                geno_habitat_datas::address.eq(any(geno_rental_agreements::table
                    .filter(geno_rental_agreements::alchemist.is_not_null())
                    .select(geno_rental_agreements::habitat_address))),
            );
        }

        if has_alchemist == Some(false) {
            query = query.filter(
                geno_habitat_datas::address.ne(any(geno_rental_agreements::table
                    .filter(geno_rental_agreements::alchemist.is_not_null())
                    .select(geno_rental_agreements::habitat_address))),
            );
        }

        if let Some(rental_open_market) = rental_open_market {
            query = query.filter(
                geno_habitat_datas::address.eq(any(geno_rental_agreements::table
                    .filter(geno_rental_agreements::open_market.eq(rental_open_market))
                    .select(geno_rental_agreements::habitat_address))),
            );

            count = false;
        }

        if sort {
            // If someone has a less stupid way to do this, I'm all ears
            match (sort_field, sort_desc) {
                (HabitatSortField::Address, false) => {
                    query = query.order_by(geno_habitat_datas::address.asc());
                },
                (HabitatSortField::Address, true) => {
                    query = query.order_by(geno_habitat_datas::address.desc());
                },
                (HabitatSortField::Level, false) => {
                    query = query.order_by(geno_habitat_datas::level.asc());
                },
                (HabitatSortField::Level, true) => {
                    query = query.order_by(geno_habitat_datas::level.desc());
                },
                (HabitatSortField::Lifespan, false) => {
                    query = query.order_by(geno_habitat_datas::expiry_timestamp.asc());
                },
                (HabitatSortField::Lifespan, true) => {
                    query = query.order_by(geno_habitat_datas::expiry_timestamp.desc());
                },
                (HabitatSortField::KiHarvested, false) => {
                    query = query.order_by(geno_habitat_datas::ki_harvested.asc());
                },
                (HabitatSortField::KiHarvested, true) => {
                    query = query.order_by(geno_habitat_datas::ki_harvested.desc());
                },
                (HabitatSortField::CrystalsRefined, false) => {
                    query = query.order_by(geno_habitat_datas::crystals_refined.asc());
                },
                (HabitatSortField::CrystalsRefined, true) => {
                    query = query.order_by(geno_habitat_datas::crystals_refined.desc());
                },
                (HabitatSortField::TotalKiHarvested, false) => {
                    query = query.order_by(geno_habitat_datas::total_ki_harvested.asc());
                },
                (HabitatSortField::TotalKiHarvested, true) => {
                    query = query.order_by(geno_habitat_datas::total_ki_harvested.desc());
                },
                (HabitatSortField::KiAvailableToHarvest, false) => {
                    query = query.order_by(geno_habitat_datas::ki_available_to_harvest.asc());
                },
                (HabitatSortField::KiAvailableToHarvest, true) => {
                    query = query.order_by(geno_habitat_datas::ki_available_to_harvest.desc());
                },
            }
        }

        (query, count)
    };

    let (query, count) = build_query(true);

    let count = if count {
        // I can't figure out any way to clone or borrow a boxed select statement.
        let (query, _) = build_query(false);

        Some(
            query
                .count()
                .get_result(conn)
                .context("Failed to count Genopets habitats")?,
        )
    } else {
        None
    };

    Ok((
        query
            .limit(limit)
            .offset(offset)
            .load(conn)
            .context("Failed to load Genopets habitats")?,
        count,
    ))
}
