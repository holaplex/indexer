use std::collections::HashMap;

use indexer_core::db::{
    delete, insert_into,
    models::{StoreConfigJson, StoreCreator},
    tables::{store_config_jsons, store_creators},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::Client;
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Creator {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub owner: String,
    pub auction_house: String,
    pub store: String,
    pub store_config: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Upload {
    pub url: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub ty: Option<String>,
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

    debug!(
        "Attempting to fetch store config: {:?}, with uri: {:?}",
        config_key, uri_str
    );

    // TODO: parse failure shouldn't be an error, this stuff will be unstructured
    let json = client
        .http(|h| async move {
            h.get(url)
                .timeout(client.timeout())
                .send()
                .await?
                .json::<SettingUri>()
                .await
        })
        .await
        .context("Store config JSON request failed")?;

    let addr = bs58::encode(config_key).into_string();

    if addr != json.address.store_config {
        info!("store config address does not match setting uri JSON config address");
        return Ok(());
    }

    let row = StoreConfigJson {
        config_address: Owned(addr.clone()),
        name: Owned(json.meta.name),
        description: Owned(json.meta.description),
        logo_url: Owned(json.theme.logo.url),
        banner_url: Owned(json.theme.banner.url),
        subdomain: Owned(json.subdomain),
        owner_address: Owned(json.address.owner),
        auction_house_address: Owned(json.address.auction_house),
        store_address: Some(Owned(json.address.store)),
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
        .context("failed to insert store config json")?;

    if let Some(creators) = json.creators {
        client
            .db()
            .run(move |db| {
                let remove_creators = store_creators::table
                    .filter(store_creators::store_config_address.eq(addr.clone()))
                    .select(store_creators::creator_address)
                    .get_results::<String>(db)
                    .unwrap_or_else(|_| Vec::new())
                    .into_iter()
                    .filter(|address| !creators.clone().into_iter().any(|c| &c.address == address))
                    .collect::<Vec<_>>();

                db.build_transaction().read_write().run(|| {
                    delete(
                        store_creators::table
                            .filter(store_creators::creator_address.eq(any(remove_creators)))
                            .filter(store_creators::store_config_address.eq(addr.clone())),
                    )
                    .execute(db)?;

                    creators.into_iter().try_for_each(|creator| {
                        let row = StoreCreator {
                            store_config_address: Owned(addr.clone()),
                            creator_address: Owned(creator.address),
                        };

                        insert_into(store_creators::table)
                            .values(&row)
                            .on_conflict((
                                store_creators::store_config_address,
                                store_creators::creator_address,
                            ))
                            .do_update()
                            .set(&row)
                            .execute(db)
                            .map(|_| ())
                    })
                })
            })
            .await
            .context("failed to insert store creator")?;
    }

    Ok(())
}
