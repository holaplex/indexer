use indexer_core::{
    db::{
        insert_into,
        models::{
            SettingsUriJson, StoreConfig as DbStoreConfig, Storefrontv2,
            WhitelistedCreator as DbWhitelistedCreator,
        },
        tables::{settings_uri_jsons, storefrontsv2, storefrontsv2_configs, whitelisted_creators},
    },
    hash::HashMap,
    prelude::*,
    pubkeys::find_store_config,
};
use mpl_metaplex::{
    state::{
        Key, Store, StoreConfig, WhitelistedCreator, MAX_STORE_CONFIG_V1_SIZE, MAX_STORE_SIZE,
        MAX_WHITELISTED_CREATOR_SIZE,
    },
    utils::try_from_slice_checked,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{prelude::*, Client};
const STORE: u8 = Key::StoreV1 as u8;
const STORE_CONFIG: u8 = Key::StoreConfigV1 as u8;
const WHITELISTED_CREATOR: u8 = Key::WhitelistedCreatorV1 as u8;

#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    pub creator_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SettingUri {
    meta: Metadata,
    logo_url: String,
    banner_url: String,
    subdomain: String,
    owner_address: String,
    auction_house_address: String,
    creators: Option<Vec<Creator>>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// process the settings_uri URL
async fn process_settings_uri(client: &Client, uri: String, pda: String) -> Result<()> {
    println!("process_settings_uri called!");
    let url = Url::parse(&uri).context("Failed to parse settings_uri URL!")?;
    let http_client = reqwest::Client::new();
    // get the data and parse it into the SettingUri struct
    let json = http_client
        .get(url.clone())
        .send()
        .await
        .context("Metadata JSON request failed")?
        .json::<SettingUri>()
        .await
        .context("Failed to parse metadata JSON")?;
    let row = SettingsUriJson {
        store_config_pda: Owned(pda),
        name: Owned(json.meta.name),
        description: Owned(json.meta.description),
        logo_url: Owned(json.logo_url),
        banner_url: Owned(json.banner_url),
        subdomain: Owned(json.subdomain),
        owner_address: Owned(json.owner_address),
        auction_house_address: Owned(json.auction_house_address),
    };
    // insert into the database
    client
        .db(move |db| {
            insert_into(settings_uri_jsons::table)
                .values(&row)
                .on_conflict(settings_uri_jsons::store_config_pda)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert metadata")?;

    println!("inserted into settings_uri_jsons table!");
    Ok(())
}

/// Deserialize the `StoreConfig` account and insert the record into `storefrontsv2_configs` table
async fn process_store_config(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    println!("process_store_config called!");
    let config: StoreConfig =
        try_from_slice_checked(&data, Key::StoreConfigV1, MAX_STORE_CONFIG_V1_SIZE)
            .context("failed to parse store config!")?;
    dbg!("{:?}", &config.settings_uri);
    let addr = bs58::encode(key).into_string();
    if config.settings_uri.is_some() {
        process_settings_uri(client, config.settings_uri.clone().unwrap(), addr.clone()).await;
    }
    let row: DbStoreConfig = DbStoreConfig {
        address: Owned(addr),
        settings_uri: config.settings_uri.map(Owned),
    };

    client
        .db(move |db| {
            insert_into(storefrontsv2_configs::table)
                .values(&row)
                .on_conflict(storefrontsv2_configs::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert store!")?;
    println!("inserted into storefrontsv2_configs table!");
    Ok(())
}

/// Deserialize `WhitelistedCreator` account and insert the record into `whitelisted_creators` table
async fn process_whitelisted_creator(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    println!("process_whitelisted_creator called!");
    let creator: WhitelistedCreator = try_from_slice_checked(
        &data,
        Key::WhitelistedCreatorV1,
        MAX_WHITELISTED_CREATOR_SIZE,
    )
    .context("failed to parse whitelisted creators!")?;
    let row: DbWhitelistedCreator = DbWhitelistedCreator {
        address: Owned(bs58::encode(key).into_string()),
        creator_address: Owned(bs58::encode(creator.address).into_string()),
        activated: creator.activated,
    };

    client
        .db(move |db| {
            insert_into(whitelisted_creators::table)
                .values(&row)
                .on_conflict((
                    whitelisted_creators::address,
                    whitelisted_creators::creator_address,
                ))
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert whitelisted creator!")?;
    println!("inserted into whitelisted_creators table!");
    Ok(())
}

/// Deserialize `Store` account and insert the record into `storefrontsv2` table
async fn process_store(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    println!("process_store called!");
    let store: Store = try_from_slice_checked(&data, Key::StoreV1, MAX_STORE_SIZE)
        .context("failed to parse storefront!")?;
    let (store_config_pda, _) = find_store_config(&key);
    let row: Storefrontv2 = Storefrontv2 {
        store_address: Owned(bs58::encode(key).into_string()),
        public: store.public,
        auction_program: Owned(bs58::encode(store.auction_program).into_string()),
        token_vault_program: Owned(bs58::encode(store.token_vault_program).into_string()),
        token_metadata_program: Owned(bs58::encode(store.token_metadata_program).into_string()),
        token_program: Owned(bs58::encode(store.token_program).into_string()),
        store_config_pda: Owned(bs58::encode(store_config_pda).into_string()),
    };

    client
        .db(move |db| {
            insert_into(storefrontsv2::table)
                .values(&row)
                .on_conflict(storefrontsv2::store_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert storefrontv2!")?;
    println!("inserted into storefrontsv2 table!");
    Ok(())
}

pub async fn process(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    // get the first byte which is a key in accounts owned by metaplex program
    // call the function according to the key
    let first_byte = data[0] as u8;
    info!("{:?}", first_byte);
    match first_byte {
        STORE => process_store(client, key, data).await,
        STORE_CONFIG => process_store_config(client, key, data).await,
        WHITELISTED_CREATOR => process_whitelisted_creator(client, key, data).await,
        _ => Ok(()),
    }
}
