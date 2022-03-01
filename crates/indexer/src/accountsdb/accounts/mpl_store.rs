use indexer_core::{
    db::{
        insert_into,
        models::{
            Store as DbStore, StoreConfig as DbStoreConfig,
            WhitelistedCreator as DbWhitelistedCreator,
        },
        tables::{store_configs, stores, whitelisted_creators},
    },
    prelude::*,
    pubkeys::find_store_config,
};
use metaplex::state::{Store, WhitelistedCreator};
use mpl_metaplex::state::StoreConfig;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_config(
    client: &Client,
    key: Pubkey,
    config: StoreConfig,
) -> Result<()> {
    trace!("{:?}", &config.settings_uri);

    let addr = bs58::encode(key).into_string();
    if let Some(ref uri) = config.settings_uri {
        debug!("dispatching store config: {:?}", uri);
        client
            .dispatch_store_config(key, uri.clone())
            .await
            .context("Failed to dispatch store config job")?;
    }

    let row = DbStoreConfig {
        address: Owned(addr),
        settings_uri: config.settings_uri.map(Owned),
    };

    client
        .db()
        .run(move |db| {
            insert_into(store_configs::table)
                .values(&row)
                .on_conflict(store_configs::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert store")?;
    Ok(())
}

pub(crate) async fn process_whitelisted_creator(
    client: &Client,
    key: Pubkey,
    creator: WhitelistedCreator,
) -> Result<()> {
    let row = DbWhitelistedCreator {
        address: Owned(bs58::encode(key).into_string()),
        creator_address: Owned(bs58::encode(creator.address).into_string()),
        activated: creator.activated,
    };

    client
        .db()
        .run(move |db| {
            insert_into(whitelisted_creators::table)
                .values(&row)
                .on_conflict(whitelisted_creators::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert whitelisted creator")?;
    Ok(())
}

pub(crate) async fn process(client: &Client, key: Pubkey, store: Store) -> Result<()> {
    let (config_address, _bump) = find_store_config(&key);

    let row = DbStore {
        address: Owned(bs58::encode(key).into_string()),
        public: store.public,
        config_address: Owned(bs58::encode(config_address).into_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(stores::table)
                .values(&row)
                .on_conflict(stores::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert storefrontv2!")?;
    Ok(())
}
