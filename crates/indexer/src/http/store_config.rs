use std::collections::HashMap;

use indexer_core::db::{insert_into, models::StoreConfigJson, tables::store_config_jsons};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::Client;
use crate::prelude::*;

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
pub struct Address {
    pub owner: String,
    pub auctionHouse: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Upload {
    pub url: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Theme {
    pub banner: Upload,
    pub logo: Upload,
}

#[derive(Serialize, Deserialize, Debug)]
struct SettingUri {
    meta: Metadata,
    theme: Theme,
    subdomain: String,
    address: Address,
    creators: Option<Vec<Creator>>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

pub async fn process(client: &Client, config_key: Pubkey, uri_str: String) -> Result<()> {
    let url = Url::parse(&uri_str).context("Couldn't parse store config URL")?;

    let http_client = reqwest::Client::new();

    debug!(
        "attempting to process storeconfig: {:?}, with uri: {:?}",
        config_key, uri_str
    );

    // TODO: parse failure shouldn't be an error, this stuff will be unstructured
    let json = http_client
        .get(url.clone())
        .send()
        .await
        .context("Metadata JSON request failed")?
        .json::<SettingUri>()
        .await
        .context("Failed to parse metadata JSON")?;

    let addr = bs58::encode(config_key).into_string();
    let row = StoreConfigJson {
        config_address: Owned(addr),
        name: Owned(json.meta.name),
        description: Owned(json.meta.description),
        logo_url: Owned(json.theme.logo.url),
        banner_url: Owned(json.theme.banner.url),
        subdomain: Owned(json.subdomain),
        owner_address: Owned(json.address.owner),
        auction_house_address: Owned(json.address.auctionHouse),
    };

    client
        .db()
        .run(move |db| {
            insert_into(store_config_jsons::table)
                .values(&row)
                .on_conflict(store_config_jsons::config_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert metadata")?;

    Ok(())
}
