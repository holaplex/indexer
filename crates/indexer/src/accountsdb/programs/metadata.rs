use mpl_token_metadata::{
    state::{
        Edition, Key, MasterEditionV1, MasterEditionV2, Metadata, MAX_EDITION_LEN,
        MAX_MASTER_EDITION_LEN, MAX_METADATA_LEN,
    },
    utils::try_from_slice_checked,
};

use super::{
    accounts::{edition, metadata},
    AccountUpdate, Client,
};
use crate::prelude::*;

const METADATA: u8 = Key::MetadataV1 as u8;
const EDITION_V1: u8 = Key::EditionV1 as u8;
const MASTER_EDITION_V1: u8 = Key::MasterEditionV1 as u8;
const MASTER_EDITION_V2: u8 = Key::MasterEditionV2 as u8;

async fn process_metadata(client: &Client, update: AccountUpdate) -> Result<()> {
    let meta: Metadata = try_from_slice_checked(&update.data, Key::MetadataV1, MAX_METADATA_LEN)
        .context("Failed to parse metadata")?;

    metadata::process(client, update.key, meta).await
}

async fn process_edition(client: &Client, update: AccountUpdate) -> Result<()> {
    let edition: Edition = try_from_slice_checked(&update.data, Key::EditionV1, MAX_EDITION_LEN)
        .context("Failed to parse edition data")?;

    edition::process(client, update.key, edition).await
}

async fn process_master_edition_v1(client: &Client, update: AccountUpdate) -> Result<()> {
    let MasterEditionV1 {
        key,
        supply,
        max_supply,
        ..
    } = try_from_slice_checked(&update.data, Key::MasterEditionV1, MAX_MASTER_EDITION_LEN)
        .context("Failed to parse master edition v1 data")?;

    let master_edition = MasterEditionV2 {
        key,
        supply,
        max_supply,
    };

    edition::process_master(client, update.key, master_edition).await
}

async fn process_master_edition_v2(client: &Client, update: AccountUpdate) -> Result<()> {
    let master_edition: MasterEditionV2 =
        try_from_slice_checked(&update.data, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
            .context("Failed to parse master edition v2 data")?;

    edition::process_master(client, update.key, master_edition).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let first_byte = update.data[0] as u8;

    match first_byte {
        METADATA => process_metadata(client, update).await,
        EDITION_V1 => process_edition(client, update).await,
        MASTER_EDITION_V1 => process_master_edition_v1(client, update).await,
        MASTER_EDITION_V2 => process_master_edition_v2(client, update).await,
        b => {
            trace!("Unhandled metadata key byte {:02x}", b);

            Ok(())
        },
    }
}
