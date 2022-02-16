use metaplex::{
    state::{Key, Store, WhitelistedCreator, MAX_STORE_SIZE, MAX_WHITELISTED_CREATOR_SIZE},
    utils::try_from_slice_checked,
};
use mpl_metaplex::{
    state::{
        Key as MplKey, StoreConfig as MplStoreConfig,
        MAX_STORE_CONFIG_V1_SIZE as MPL_MAX_STORE_CONFIG_V1_SIZE,
        MAX_STORE_SIZE as MPL_MAX_STORE_SIZE,
        MAX_WHITELISTED_CREATOR_SIZE as MPL_MAX_WHITELISTED_CREATOR_SIZE,
    },
    utils::try_from_slice_checked as mpl_try_from_slice_checked,
};

use super::{accounts::mpl_store, AccountUpdate, Client};
use crate::prelude::*;

// TODO: once we switch to mpl_metaplex, remove all the MPL_ prefixes and
//       cross-package assertions

const MPL_STORE: u8 = MplKey::StoreV1 as u8;
const MPL_STORE_CONFIG: u8 = MplKey::StoreConfigV1 as u8;
const MPL_WHITELISTED_CREATOR: u8 = MplKey::WhitelistedCreatorV1 as u8;

const STORE: u8 = Key::StoreV1 as u8;
const WHITELISTED_CREATOR: u8 = Key::WhitelistedCreatorV1 as u8;

async fn process_store(client: &Client, update: AccountUpdate) -> Result<()> {
    assert_eq!(MPL_MAX_STORE_SIZE, MAX_STORE_SIZE);

    let store: Store = try_from_slice_checked(&update.data, Key::StoreV1, MAX_STORE_SIZE)
        .context("Failed to parse store data")?;

    mpl_store::process(client, update.key, store).await
}

async fn process_whitelisted_creator(client: &Client, update: AccountUpdate) -> Result<()> {
    assert_eq!(
        MPL_MAX_WHITELISTED_CREATOR_SIZE,
        MAX_WHITELISTED_CREATOR_SIZE
    );

    let creator: WhitelistedCreator = try_from_slice_checked(
        &update.data,
        Key::WhitelistedCreatorV1,
        MAX_WHITELISTED_CREATOR_SIZE,
    )
    .context("Failed to parse whitelisted creator data")?;

    mpl_store::process_whitelisted_creator(client, update.key, creator).await
}

async fn process_store_config(client: &Client, update: AccountUpdate) -> Result<()> {
    let config: MplStoreConfig = mpl_try_from_slice_checked(
        &update.data,
        MplKey::StoreConfigV1,
        MPL_MAX_STORE_CONFIG_V1_SIZE,
    )
    .context("Failed to parse store data")?;

    mpl_store::process_config(client, update.key, config).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let first_byte = update.data[0] as u8;

    assert_eq!(MPL_STORE, STORE);
    assert_eq!(MPL_WHITELISTED_CREATOR, WHITELISTED_CREATOR);

    match first_byte {
        STORE => process_store(client, update).await,
        WHITELISTED_CREATOR => process_whitelisted_creator(client, update).await,
        MPL_STORE_CONFIG => process_store_config(client, update).await,
        b => {
            debug!("Unhandled metadata key byte {:02x}", b);

            Ok(())
        },
    }
}
