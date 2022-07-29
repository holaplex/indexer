//! Query utilities for Genopets

use std::ops::Bound;

use chrono::prelude::*;
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

/// Input parameters for the [`list_habitats`] query.
#[derive(Debug)]
pub struct ListHabitatOptions<W, H> {
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
    /// Limit the number of returned rows
    pub limit: i64,
    /// Skip the first `n` resulting rows
    pub offset: i64,
}

/// List the Genopets `HabitatData` accounts matching the given query parameters
///
/// # Errors
/// This function fails if the underlying query returns an error.
pub fn list_habitats<W: ToSql<Text, Pg> + ToSql<Nullable<Text>, Pg>, H: ToSql<Text, Pg>>(
    conn: &Connection,
    opts: ListHabitatOptions<W, H>,
) -> Result<Vec<GenoHabitatData<'static>>> {
    let mut query = geno_habitat_datas::table
        .select(geno_habitat_datas::all_columns)
        .into_boxed();

    let ListHabitatOptions {
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
        limit,
        offset,
    } = opts;

    if let Some(owners) = owners {
        query = query.filter(
            geno_habitat_datas::habitat_mint.eq(any(current_metadata_owners::table
                .filter(current_metadata_owners::owner_address.eq(any(owners)))
                .select(current_metadata_owners::mint_address))),
        );
    }

    if let Some(renters) = renters {
        query = query.filter(
            geno_habitat_datas::address.eq(any(geno_rental_agreements::table
                .filter(geno_rental_agreements::alchemist.eq(any(renters)))
                .select(geno_rental_agreements::habitat_address))),
        );
    }

    if let Some(harvesters) = harvesters {
        query = query.filter(geno_habitat_datas::harvester.eq(any(harvesters)));
    }

    if let Some(genesis) = genesis {
        query = query.filter(geno_habitat_datas::genesis.eq(genesis));
    }

    if let Some(elements) = elements {
        query = query.filter(geno_habitat_datas::element.eq(any(elements)));
    }

    query = handle_range(query, geno_habitat_datas::level, levels);
    query = handle_range(query, geno_habitat_datas::sequence, sequences);

    if let Some(guilds) = guilds {
        query = query.filter(geno_habitat_datas::guild.eq(any(guilds)));
    }

    query = handle_range(query, geno_habitat_datas::durability, durabilities);

    {
        let (min, max) = expiries;
        let min = min.map(|m| m.naive_utc());
        let max = max.map(|m| m.naive_utc());

        query = handle_range(query, geno_habitat_datas::expiry_timestamp, (min, max));
    }

    query
        .limit(limit)
        .offset(offset)
        .load(conn)
        .context("Failed to load Genopets habitats")
}
