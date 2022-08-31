use indexer_core::{
    db::{models, queries::genopets},
    meilisearch::IndirectMetadataDocument,
};
use objects::{nft::Nft, wallet::Wallet};
use scalars::{markers::TokenMint, PublicKey, I64};

use super::prelude::*;

#[derive(Debug, GraphQLInputObject)]
/// Input parameters for the `genoHabitatsCounted` query
pub struct GenoHabitatsParams {
    /// Filter by habitat NFT addresses
    pub mints: Option<Vec<PublicKey<TokenMint>>>,
    /// Filter by habitat NFT owners
    pub owners: Option<Vec<PublicKey<Wallet>>>,
    /// Filter by renter addresses
    pub renters: Option<Vec<PublicKey<Wallet>>>,
    /// Filter by harvester addresses
    pub harvesters: Option<Vec<String>>,
    /// Filter by genesis (or non-genesis)
    pub genesis: Option<bool>,
    /// Filter by elements
    pub elements: Option<Vec<i32>>,
    /// Minimum habitat level to return
    pub min_level: Option<i32>,
    /// Maximum habitat level to return
    pub max_level: Option<i32>,
    /// Minimum habitat sequence number to return
    pub min_sequence: Option<i32>,
    /// Maximum habitat sequence number to return
    pub max_sequence: Option<i32>,
    /// Filter by guild IDs
    pub guilds: Option<Vec<i32>>,
    /// Minimum habitat durability to return
    pub min_durability: Option<i32>,
    /// Maximum habitat durability to return
    pub max_durability: Option<i32>,
    /// Minimum habitat expiry timestamp to return
    pub min_expiry: Option<DateTime<Utc>>,
    /// Maximum habitat expiry timestamp to return
    pub max_expiry: Option<DateTime<Utc>>,
    /// Filter by open (or closed) market
    pub harvester_open_market: Option<bool>,
    /// Filter by rental open (or closed) market
    pub rental_open_market: Option<bool>,
    /// Filter habitats by a fuzzy text-search query
    pub term: Option<String>,
    /// Field to sort results by (default `ADDRESS`)
    pub sort_field: Option<GenoHabitatSortField>,
    /// True to sort results in descending order (default false)
    pub sort_desc: Option<bool>,
    /// Maximum number of results to return (max 250)
    pub limit: i32,
    /// Pagination offset
    pub offset: i32,
}

impl GenoHabitatsParams {
    pub async fn into_db_opts(
        self,
        ctx: &AppContext,
    ) -> juniper::FieldResult<
        genopets::ListHabitatOptions<PublicKey<TokenMint>, PublicKey<Wallet>, String>,
    > {
        use std::ops::Bound;

        fn make_range<T>(min: Option<T>, max: Option<T>) -> (Bound<T>, Bound<T>) {
            (
                min.map_or(Bound::Unbounded, Bound::Included),
                max.map_or(Bound::Unbounded, Bound::Included),
            )
        }

        let GenoHabitatsParams {
            mints,
            owners,
            renters,
            harvesters,
            genesis,
            elements,
            min_level,
            max_level,
            min_sequence,
            max_sequence,
            guilds,
            min_durability,
            max_durability,
            min_expiry,
            max_expiry,
            harvester_open_market,
            rental_open_market,
            term,
            sort_field,
            sort_desc,
            limit,
            offset,
        } = self;

        let mints = match (mints, term) {
            (m, None) => m,
            (None, Some(ref t)) => Some({
                ctx.shared
                    .search
                    .index("geno_habitats")
                    .search()
                    .with_query(t)
                    .with_limit(ctx.shared.pre_query_search_limit)
                    .execute::<IndirectMetadataDocument>()
                    .await
                    .context("Failed to load search results for Genopets habitats")?
                    .hits
                    .into_iter()
                    .map(|r| r.result.mint_address.into())
                    .collect()
            }),
            (Some(_), Some(_)) => {
                return Err(FieldError::new(
                    "The mints and term parameters cannot be combined",
                    graphql_value!(["mints", "term"]),
                ));
            },
        };

        if limit > 250 {
            return Err(FieldError::new(
                "The query limit cannot be higher than 250",
                graphql_value!(limit),
            ));
        }

        Ok(genopets::ListHabitatOptions {
            mints,
            owners,
            renters,
            harvesters,
            genesis,
            elements: elements
                .map(|e| e.into_iter().map(TryInto::try_into).collect())
                .transpose()
                .context("Failed to convert elements parameter")?,
            levels: make_range(
                min_level
                    .map(TryInto::try_into)
                    .transpose()
                    .context("Failed to convert min level")?,
                max_level
                    .map(TryInto::try_into)
                    .transpose()
                    .context("Failed to convert max level")?,
            ),
            sequences: make_range(min_sequence.map(Into::into), max_sequence.map(Into::into)),
            guilds,
            durabilities: make_range(min_durability, max_durability),
            expiries: make_range(min_expiry, max_expiry),
            harvester_open_market,
            rental_open_market,
            sort_field: sort_field.unwrap_or(GenoHabitatSortField::Address).into(),
            sort_desc: sort_desc.unwrap_or(false),
            limit: limit.into(),
            offset: offset.into(),
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "A list of Genopets habitats", Context = AppContext)]
pub struct GenoHabitatList {
    pub habitats: Vec<GenoHabitat>,
    pub total_count_hint: Option<I64>,
}

impl From<genopets::HabitatList> for GenoHabitatList {
    fn from((habitats, total_count_hint): genopets::HabitatList) -> Self {
        Self {
            habitats: habitats.into_iter().map(Into::into).collect(),
            total_count_hint: total_count_hint.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Copy, GraphQLEnum)]
/// Input sorting parameter for the `genoHabitatsCounted` query
pub enum GenoHabitatSortField {
    /// Sort by the `address` field
    Address,
    /// Sort by the `level` field
    Level,
    /// Sort by the `expiryTimestamp` field
    Lifespan,
    /// Sort by the `kiHarvested` field
    KiHarvested,
    /// Sort by the `crystalsRefined` field
    CrystalsRefined,
}

impl From<GenoHabitatSortField> for genopets::HabitatSortField {
    fn from(field: GenoHabitatSortField) -> Self {
        match field {
            GenoHabitatSortField::Address => Self::Address,
            GenoHabitatSortField::Level => Self::Level,
            GenoHabitatSortField::Lifespan => Self::Lifespan,
            GenoHabitatSortField::KiHarvested => Self::KiHarvested,
            GenoHabitatSortField::CrystalsRefined => Self::CrystalsRefined,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct GenoHabitat {
    pub address: PublicKey<GenoHabitat>,
    pub habitat_mint: PublicKey<TokenMint>,
    pub level: i32,
    pub element: i32,
    pub genesis: bool,
    pub renewal_timestamp: DateTime<Utc>,
    pub expiry_timestamp: DateTime<Utc>,
    pub next_day_timestamp: DateTime<Utc>,
    pub crystals_refined: i32,
    pub harvester: String,
    pub ki_harvested: I64,
    pub seeds_spawned: bool,
    pub is_sub_habitat: bool,
    pub parent_habitat: Option<PublicKey<GenoHabitat>>,
    pub sub_habitats: Vec<PublicKey<GenoHabitat>>,
    pub harvester_royalty_bips: i32,
    pub harvester_open_market: bool,
    pub total_ki_harvested: I64,
    pub total_crystals_refined: I64,
    pub terraforming_habitat: Option<PublicKey<TokenMint>>,
    pub active: bool,
    pub durability: i32,
    pub habitats_terraformed: i32,
    pub sequence: I64,
    pub guild: Option<i32>,
    pub sub_habitat_cooldown_timestamp: DateTime<Utc>,
    pub harvester_settings_cooldown_timestamp: DateTime<Utc>,
}

#[graphql_object(Context = AppContext)]
impl GenoHabitat {
    pub fn address(&self) -> &PublicKey<GenoHabitat> {
        &self.address
    }

    pub fn habitat_mint(&self) -> &PublicKey<TokenMint> {
        &self.habitat_mint
    }

    pub fn level(&self) -> &i32 {
        &self.level
    }

    pub fn element(&self) -> &i32 {
        &self.element
    }

    pub fn genesis(&self) -> &bool {
        &self.genesis
    }

    pub fn renewal_timestamp(&self) -> &DateTime<Utc> {
        &self.renewal_timestamp
    }

    pub fn expiry_timestamp(&self) -> &DateTime<Utc> {
        &self.expiry_timestamp
    }

    pub fn next_day_timestamp(&self) -> &DateTime<Utc> {
        &self.next_day_timestamp
    }

    pub fn crystals_refined(&self) -> &i32 {
        &self.crystals_refined
    }

    pub fn harvester(&self) -> &String {
        &self.harvester
    }

    pub fn ki_harvested(&self) -> &I64 {
        &self.ki_harvested
    }

    pub fn seeds_spawned(&self) -> &bool {
        &self.seeds_spawned
    }

    pub fn is_sub_habitat(&self) -> &bool {
        &self.is_sub_habitat
    }

    pub fn parent_habitat(&self) -> &Option<PublicKey<GenoHabitat>> {
        &self.parent_habitat
    }

    pub fn sub_habitats(&self) -> &Vec<PublicKey<GenoHabitat>> {
        &self.sub_habitats
    }

    pub fn harvester_royalty_bips(&self) -> &i32 {
        &self.harvester_royalty_bips
    }

    pub fn harvester_open_market(&self) -> &bool {
        &self.harvester_open_market
    }

    pub fn total_ki_harvested(&self) -> &I64 {
        &self.total_ki_harvested
    }

    pub fn total_crystals_refined(&self) -> &I64 {
        &self.total_crystals_refined
    }

    pub fn terraforming_habitat(&self) -> &Option<PublicKey<TokenMint>> {
        &self.terraforming_habitat
    }

    pub fn active(&self) -> &bool {
        &self.active
    }

    pub fn durability(&self) -> &i32 {
        &self.durability
    }

    pub fn habitats_terraformed(&self) -> &i32 {
        &self.habitats_terraformed
    }

    pub fn sequence(&self) -> &I64 {
        &self.sequence
    }

    pub fn guild(&self) -> &Option<i32> {
        &self.guild
    }

    pub fn sub_habitat_cooldown_timestamp(&self) -> &DateTime<Utc> {
        &self.sub_habitat_cooldown_timestamp
    }

    pub fn harvester_settings_cooldown_timestamp(&self) -> &DateTime<Utc> {
        &self.harvester_settings_cooldown_timestamp
    }

    pub async fn rental_agreement(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<GenoRentalAgreement>> {
        ctx.geno_rental_agreement_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn sub_habitat_data(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Vec<Option<GenoHabitat>>> {
        future::join_all(
            self.sub_habitats
                .iter()
                .map(|a| ctx.geno_habitat_loader.load(a.clone())),
        )
        .await
        .into_iter()
        .collect::<Result<_, _>>()
        .map_err(Into::into)
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_by_mint_loader
            .load(self.habitat_mint.clone())
            .await
            .map_err(Into::into)
    }
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
            harvester_bytes: _,
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
            harvester_settings_cooldown_timestamp,
            slot: _,
            write_version: _,
            harvester,
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
            harvester: harvester.into_owned(),
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
            harvester_settings_cooldown_timestamp: DateTime::from_utc(
                harvester_settings_cooldown_timestamp,
                Utc,
            ),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct GenoRentalAgreement {
    pub habitat_address: PublicKey<GenoHabitat>,
    pub alchemist: Option<PublicKey<Wallet>>,
    pub rental_period: I64,
    pub rent: I64,
    pub rent_token: PublicKey<TokenMint>,
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
