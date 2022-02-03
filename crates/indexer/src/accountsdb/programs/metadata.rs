use metaplex_token_metadata::{
    state::{
        Edition, Key, MasterEditionV1, MasterEditionV2, Metadata, MAX_EDITION_LEN,
        MAX_MASTER_EDITION_LEN, MAX_METADATA_LEN,
    },
    utils::try_from_slice_checked,
};

use super::accounts::{edition, metadata};
use crate::{prelude::*, Client};

const METADATA: u8 = Key::MetadataV1 as u8;
const EDITION_V1: u8 = Key::EditionV1 as u8;
const MASTER_EDITION_V1: u8 = Key::MasterEditionV1 as u8;
// const RESERVATION_LIST_V1: u8 = Key::ReservationListV1 as u8;
// const RESERVATION_LIST_V2: u8 = Key::ReservationListV2 as u8;
const MASTER_EDITION_V2: u8 = Key::MasterEditionV2 as u8;
// const EDITION_MARKER: u8 = Key::EditionMarker as u8;
// const UNINITIALIZED_BYTE: u8 = Key::Uninitialized as u8;

async fn process_metadata(client: &Client, meta_key: Pubkey, data: Vec<u8>) -> Result<()> {
    let meta: Metadata = try_from_slice_checked(&data, Key::MetadataV1, MAX_METADATA_LEN)
        .context("Failed to parse metadata")?;

    metadata::process(client, meta_key, &meta).await
}

async fn process_edition(client: &Client, edition_key: Pubkey, data: Vec<u8>) -> Result<()> {
    let edition: Edition = try_from_slice_checked(&data, Key::EditionV1, MAX_EDITION_LEN)
        .context("Failed to parse edition data")?;

    edition::process(client, edition_key, &edition).await
}

async fn process_master_edition_v1(
    client: &Client,
    master_key: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let MasterEditionV1 {
        key,
        supply,
        max_supply,
        ..
    } = try_from_slice_checked(&data, Key::MasterEditionV1, MAX_MASTER_EDITION_LEN)
        .context("Failed to parse master edition v1 data")?;

    let master_edition = MasterEditionV2 {
        key,
        supply,
        max_supply,
    };

    edition::process_master(client, master_key, &master_edition).await
}

async fn process_master_edition_v2(
    client: &Client,
    master_key: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let master_edition: MasterEditionV2 =
        try_from_slice_checked(&data, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
            .context("Failed to parse master edition v2 data")?;

    edition::process_master(client, master_key, &master_edition).await
}

pub(crate) async fn process(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    let first_byte = data[0] as u8;

    match first_byte {
        _ if first_byte == METADATA => process_metadata(client, key, data).await,
        _ if first_byte == EDITION_V1 => process_edition(client, key, data).await,
        _ if first_byte == MASTER_EDITION_V1 => process_master_edition_v1(client, key, data).await,
        _ if first_byte == MASTER_EDITION_V2 => process_master_edition_v2(client, key, data).await,
        b => {
            debug!("Unhandled metadata key byte {:02x}", b);

            Ok(())
        },
    }
}
