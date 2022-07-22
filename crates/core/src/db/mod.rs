//! Interface with the indexer database

pub mod custom_types;
pub mod models;
pub mod queries;
#[allow(missing_docs, unused_imports)]
mod schema;

pub mod tables {
    //! Diesel schema DSLs

    pub use super::schema::*;
}

pub use diesel::{
    backend::Backend,
    debug_query, delete, expression, insert_into,
    pg::{
        upsert::{excluded, on_constraint},
        Pg,
    },
    query_dsl,
    result::{DatabaseErrorKind, Error},
    select, serialize, sql_query, sql_types, update, Queryable,
};
use diesel::{pg, r2d2};
pub use diesel_full_text_search::{
    websearch_to_tsquery, TsQuery, TsQueryExtensions, TsVector, TsVectorExtensions,
};

use crate::prelude::*;

embed_migrations!("migrations");

/// Postgres database connection
pub type Connection = pg::PgConnection;
/// R2D2 connection manager for Postgres
pub type ConnectionManager = r2d2::ConnectionManager<Connection>;
/// R2D2 connection pool for Postgres
pub type Pool = r2d2::Pool<ConnectionManager>;
/// Pooled Postgres connection
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

/// Hint indicating how the database should be connected
#[derive(Debug, Clone, Copy)]
pub enum ConnectMode {
    /// Open the database for reading
    ///
    /// This will check for a `DATABASE_READ_URL` for read replicas.
    Read,
    /// Open the database for writing
    ///
    /// This will check for a `DATABASE_WRITE_URL` for a primary replica.
    Write,
}

/// Hint indicating how a returned database connection should be interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionType {
    /// The `DATABASE_URL` var was used and is likely writable
    Default,
    /// The `DATABASE_READ_URL` var was used and is not writable
    Read,
    /// The `DATABASE_WRITE_URL` var was used and should be writable
    Write,
}

/// Arguments for establishing a database connection
#[derive(Debug, clap::Args)]
pub struct ConnectArgs {
    /// Connection string for a read-only database
    #[clap(long, env, conflicts_with("database-write-url"))]
    database_read_url: Option<String>,

    /// Connection string for a writable database
    #[clap(long, env, conflicts_with("database-read-url"))]
    database_write_url: Option<String>,

    /// Fallback database connection string
    #[clap(
        long,
        env,
        required_unless_present_any(["database-read-url", "database-write-url"])
    )]
    database_url: Option<String>,
}

impl From<ConnectMode> for ConnectionType {
    fn from(mode: ConnectMode) -> Self {
        match mode {
            ConnectMode::Read => Self::Read,
            ConnectMode::Write => Self::Write,
        }
    }
}

/// Create a pooled connection to the Postgres database, using the given CLI
/// arguments and a hint indicating if the database is writable.
///
/// # Errors
/// This function fails if Diesel fails to construct a connection pool or if any
/// pending database migrations fail to run.
pub fn connect(args: ConnectArgs, mode: ConnectMode) -> Result<(Pool, ConnectionType)> {
    let ConnectArgs {
        database_read_url,
        database_write_url,
        database_url,
    } = args;

    let mode_url = match mode {
        ConnectMode::Read => database_read_url,
        ConnectMode::Write => database_write_url,
    };

    let (ty, url) = mode_url
        .map(|u| (mode.into(), u))
        .or_else(|| database_url.map(|u| (ConnectionType::Default, u)))
        .ok_or_else(|| {
            anyhow!(
                "Invalid database URL, expected a {} connection string",
                match mode {
                    ConnectMode::Read => "read-only",
                    ConnectMode::Write => "writable",
                }
            )
        })?;

    debug!("Connecting to db: {:?}", url);

    let man = ConnectionManager::new(url);
    let pool = Pool::builder()
        .max_size(num_cpus::get().try_into().unwrap_or(u32::MAX))
        .min_idle(Some(1))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .build(man)
        .context("Failed to create database connection pool")?;

    let mut out = vec![];

    if cfg!(not(debug_assertions)) && matches!(ty, ConnectionType::Default) {
        warn!("Cannot determine if database is writable; assuming yes");
    }

    if matches!(ty, ConnectionType::Read) {
        info!("Not running migrations over a read-only connection");
    } else {
        info!("Running database migrations...");
        embedded_migrations::run_with_output(
            &pool.get().context("Failed to connect to the database")?,
            &mut out,
        )
        .context("Failed to run database migrations")?;
    }

    match std::str::from_utf8(&out) {
        Ok(s) => {
            let s = s.trim();

            if !s.is_empty() {
                info!("Output from migrations:\n{}", s);
            }
        },
        Err(e) => warn!("Failed to read migration output: {}", e),
    }

    Ok((pool, ty))
}

#[cfg(test)]
pub mod test {
    use diesel::{insert_into, prelude::*};
    use uuid::Uuid;

    use super::{
        connect, models,
        schema::{
            auction_houses, listings, metadata_collection_keys, metadata_jsons, metadatas,
            purchases,
        },
        ConnectArgs,
    };
    use crate::prelude::*;

    fn initialize() -> super::Pool {
        let conn_args = ConnectArgs {
            database_read_url: None,
            database_write_url: Some(
                "postgres://postgres:holap1ex@localhost:5337/holaplex-indexer".into(),
            ),
            database_url: None,
        };
        let (pool, _) = connect(conn_args, crate::db::ConnectMode::Write)
            .expect("failed to connect to database");
        let conn = pool.get().expect("failed to get connection to database");

        let nft_a_metadata_address = Borrowed("metadata_a");
        let nft_b_metadata_address = Borrowed("metadata_b");
        let nft_c_metadata_address = Borrowed("metadata_c");
        let nft_d_metadata_address = Borrowed("metadata_d");
        let nft_d_purchase_id = Some(
            Uuid::parse_str("00000000-0000-0000-0000-000000000009").expect("failed to parse UUID"),
        );
        let collection_metadata_address = Borrowed("collection_a");
        let auction_house_address = Borrowed("auction_house_a");
        let seller_address = Borrowed("seller_a");
        let buyer_address = Borrowed("buyer_a");

        insert_into(metadata_collection_keys::table)
            .values(vec![
                models::MetadataCollectionKey {
                    metadata_address: nft_a_metadata_address.clone(),
                    collection_address: collection_metadata_address.clone(),
                    verified: true,
                },
                models::MetadataCollectionKey {
                    metadata_address: nft_b_metadata_address.clone(),
                    collection_address: collection_metadata_address.clone(),
                    verified: true,
                },
                models::MetadataCollectionKey {
                    metadata_address: nft_c_metadata_address.clone(),
                    collection_address: collection_metadata_address.clone(),
                    verified: true,
                },
                models::MetadataCollectionKey {
                    metadata_address: nft_d_metadata_address.clone(),
                    collection_address: collection_metadata_address.clone(),
                    verified: true,
                },
            ])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed metadata_collection_keys");

        insert_into(metadatas::table)
            .values(vec![
                models::Metadata {
                    address: nft_a_metadata_address.clone(),
                    name: Borrowed("nft A"),
                    symbol: Borrowed("symbol"),
                    uri: Borrowed("http://example.com/nft-a-uri"),
                    seller_fee_basis_points: 100,
                    update_authority_address: Borrowed("update authority"),
                    mint_address: collection_metadata_address.clone(),
                    primary_sale_happened: true,
                    is_mutable: false,
                    edition_nonce: None,
                    edition_pda: Borrowed("nft edition pda"),
                    token_standard: None,
                    slot: Some(0),
                    burned: false,
                },
                models::Metadata {
                    address: nft_b_metadata_address.clone(),
                    name: Borrowed("nft B"),
                    symbol: Borrowed("symbol"),
                    uri: Borrowed("http://example.com/nft-b-uri"),
                    seller_fee_basis_points: 100,
                    update_authority_address: Borrowed("update authority"),
                    mint_address: collection_metadata_address.clone(),
                    primary_sale_happened: true,
                    is_mutable: false,
                    edition_nonce: None,
                    edition_pda: Borrowed("nft edition pda"),
                    token_standard: None,
                    slot: Some(0),
                    burned: false,
                },
                models::Metadata {
                    address: nft_c_metadata_address.clone(),
                    name: Borrowed("nft C"),
                    symbol: Borrowed("symbol"),
                    uri: Borrowed("http://example.com/nft-c-uri"),
                    seller_fee_basis_points: 100,
                    update_authority_address: Borrowed("update authority"),
                    mint_address: collection_metadata_address.clone(),
                    primary_sale_happened: true,
                    is_mutable: false,
                    edition_nonce: None,
                    edition_pda: Borrowed("nft edition pda"),
                    token_standard: None,
                    slot: Some(0),
                    burned: false,
                },
                models::Metadata {
                    address: nft_d_metadata_address.clone(),
                    name: Borrowed("nft D"),
                    symbol: Borrowed("symbol"),
                    uri: Borrowed("http://example.com/nft-d-uri"),
                    seller_fee_basis_points: 100,
                    update_authority_address: Borrowed("update authority"),
                    mint_address: collection_metadata_address.clone(),
                    primary_sale_happened: true,
                    is_mutable: false,
                    edition_nonce: None,
                    edition_pda: Borrowed("nft edition pda"),
                    token_standard: None,
                    slot: Some(0),
                    burned: false,
                },
                models::Metadata {
                    address: collection_metadata_address.clone(),
                    name: Borrowed("collection name"),
                    symbol: Borrowed("symbol"),
                    uri: Borrowed("http://example.com/collection-uri"),
                    seller_fee_basis_points: 100,
                    update_authority_address: Borrowed("update authority"),
                    mint_address: Borrowed("collection mint"),
                    primary_sale_happened: true,
                    is_mutable: false,
                    edition_nonce: None,
                    edition_pda: Borrowed("collection edition pda"),
                    token_standard: None,
                    slot: Some(0),
                    burned: false,
                },
            ])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed metadatas");

        insert_into(auction_houses::table)
            .values(vec![models::AuctionHouse {
                address: auction_house_address.clone(),
                treasury_mint: Borrowed("So11111111111111111111111111111111111111112"),
                auction_house_treasury: Borrowed("treasury"),
                treasury_withdrawal_destination: Borrowed("treasury withdrawal"),
                fee_withdrawal_destination: Borrowed("fee withdrawal"),
                authority: Borrowed("auction house authority"),
                creator: Borrowed("auction house creator"),
                bump: 0,
                treasury_bump: 0,
                fee_payer_bump: 0,
                seller_fee_basis_points: 100,
                requires_sign_off: false,
                can_change_sale_price: false,
                auction_house_fee_account: Borrowed("auction house fee account"),
            }])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed auction_houses");

        insert_into(listings::table)
            .values(vec![
                models::Listing {
                    id: Some(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                            .expect("failed to parse UUID"),
                    ),
                    trade_state: Borrowed("nft_a trade state"),
                    auction_house: auction_house_address.clone(),
                    seller: seller_address.clone(),
                    metadata: nft_a_metadata_address.clone(),
                    purchase_id: None,
                    price: 1,
                    token_size: 1,
                    trade_state_bump: 0,
                    created_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    canceled_at: None,
                    slot: 0,
                    write_version: Some(0),
                },
                models::Listing {
                    id: Some(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                            .expect("failed to parse UUID"),
                    ),
                    trade_state: Borrowed("nft_b trade state"),
                    auction_house: auction_house_address.clone(),
                    seller: seller_address.clone(),
                    metadata: nft_b_metadata_address.clone(),
                    purchase_id: None,
                    price: 1,
                    token_size: 1,
                    trade_state_bump: 0,
                    created_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    canceled_at: None,
                    slot: 0,
                    write_version: Some(0),
                },
                models::Listing {
                    id: Some(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000003")
                            .expect("failed to parse UUID"),
                    ),
                    trade_state: Borrowed("nft_c trade state"),
                    auction_house: auction_house_address.clone(),
                    seller: seller_address.clone(),
                    metadata: nft_c_metadata_address.clone(),
                    purchase_id: None,
                    price: 1,
                    token_size: 1,
                    trade_state_bump: 0,
                    created_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    canceled_at: None,
                    slot: 0,
                    write_version: Some(0),
                },
                models::Listing {
                    id: Some(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000004")
                            .expect("failed to parse UUID"),
                    ),
                    trade_state: Borrowed("nft_d trade state"),
                    auction_house: auction_house_address.clone(),
                    seller: seller_address.clone(),
                    metadata: nft_d_metadata_address.clone(),
                    purchase_id: nft_d_purchase_id.clone(),
                    price: 1,
                    token_size: 1,
                    trade_state_bump: 0,
                    created_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    canceled_at: None,
                    slot: 0,
                    write_version: Some(0),
                },
            ])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed purchases");

        insert_into(purchases::table)
            .values(vec![models::Purchase {
                id: nft_d_purchase_id.clone(),
                buyer: buyer_address.clone(),
                seller: seller_address.clone(),
                auction_house: auction_house_address.clone(),
                metadata: nft_d_metadata_address.clone(),
                token_size: 1,
                price: 1,
                created_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                slot: 0,
                write_version: None,
            }])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed purchases");

        insert_into(metadata_jsons::table)
            .values(vec![
                models::MetadataJson {
                    metadata_address: nft_a_metadata_address,
                    fingerprint: Borrowed(&Vec::<u8>::new()),
                    updated_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    description: Some(Borrowed("nft A description")),
                    image: Some(Borrowed("http://example.com/nft-a-image")),
                    animation_url: Some(Borrowed("http://example.com/nft-a-animation")),
                    external_url: Some(Borrowed("http://example.com/nft-a-external")),
                    category: Some(Borrowed("nft A category")),
                    raw_content: Borrowed(
                        &serde_json::from_str("{}")
                            .expect("Failed to deserialize metadata content"),
                    ),
                    model: Some(Borrowed("model")),
                    fetch_uri: Borrowed("http://example.com/nft-a-fetch-uri"),
                    slot: 0,
                    write_version: 0,
                },
                models::MetadataJson {
                    metadata_address: nft_b_metadata_address,
                    fingerprint: Borrowed(&Vec::<u8>::new()),
                    updated_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    description: Some(Borrowed("nft B description")),
                    image: Some(Borrowed("http://example.com/nft-b-image")),
                    animation_url: Some(Borrowed("http://example.com/nft-b-animation")),
                    external_url: Some(Borrowed("http://example.com/nft-b-external")),
                    category: Some(Borrowed("nft B category")),
                    raw_content: Borrowed(
                        &serde_json::from_str("{}")
                            .expect("Failed to deserialize metadata content"),
                    ),
                    model: Some(Borrowed("model")),
                    fetch_uri: Borrowed("http://example.com/nft-b-fetch-uri"),
                    slot: 0,
                    write_version: 0,
                },
                models::MetadataJson {
                    metadata_address: nft_c_metadata_address,
                    fingerprint: Borrowed(&Vec::<u8>::new()),
                    updated_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    description: Some(Borrowed("nft C description")),
                    image: Some(Borrowed("http://example.com/nft-c-image")),
                    animation_url: Some(Borrowed("http://example.com/nft-c-animation")),
                    external_url: Some(Borrowed("http://example.com/nft-c-external")),
                    category: Some(Borrowed("nft C category")),
                    raw_content: Borrowed(
                        &serde_json::from_str("{}")
                            .expect("Failed to deserialize metadata content"),
                    ),
                    model: Some(Borrowed("model")),
                    fetch_uri: Borrowed("http://example.com/nft-c-fetch-uri"),
                    slot: 0,
                    write_version: 0,
                },
                models::MetadataJson {
                    metadata_address: nft_d_metadata_address,
                    fingerprint: Borrowed(&Vec::<u8>::new()),
                    updated_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    description: Some(Borrowed("nft C description")),
                    image: Some(Borrowed("http://example.com/nft-c-image")),
                    animation_url: Some(Borrowed("http://example.com/nft-c-animation")),
                    external_url: Some(Borrowed("http://example.com/nft-c-external")),
                    category: Some(Borrowed("nft B category")),
                    raw_content: Borrowed(
                        &serde_json::from_str("{}")
                            .expect("Failed to deserialize metadata content"),
                    ),
                    model: Some(Borrowed("model")),
                    fetch_uri: Borrowed("http://example.com/nft-c-fetch-uri"),
                    slot: 0,
                    write_version: 0,
                },
                models::MetadataJson {
                    metadata_address: collection_metadata_address,
                    fingerprint: Borrowed(&Vec::<u8>::new()),
                    updated_at: NaiveDate::from_ymd(2020, 1, 2).and_hms(0, 0, 0),
                    description: Some(Borrowed("collection description")),
                    image: Some(Borrowed("http://example.com/collection-image")),
                    animation_url: Some(Borrowed("http://example.com/collection-animation")),
                    external_url: Some(Borrowed("http://example.com/collection-external")),
                    category: Some(Borrowed("collection category")),
                    raw_content: Borrowed(
                        &serde_json::from_str("{}")
                            .expect("Failed to deserialize metadata content"),
                    ),
                    model: Some(Borrowed("model")),
                    fetch_uri: Borrowed("http://example.com/collection-fetch-uri"),
                    slot: 0,
                    write_version: 0,
                },
            ])
            .on_conflict_do_nothing()
            .execute(&conn)
            .expect("failed to seed metadata_jsons");

        pool
    }

    lazy_static::lazy_static! {
        pub static ref DATABASE: super::Pool = initialize();
    }
}
