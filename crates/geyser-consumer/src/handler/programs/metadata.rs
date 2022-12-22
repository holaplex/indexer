use metaplex_token_metadata::{
    state::{Key as MetaplexKey, Metadata as MetaplexMetadata},
    utils::try_from_slice_checked as metaplex_try_from_slice_checked,
};
use mpl_token_metadata::{
    state::{
        Creator, Data, Edition, Key, MasterEditionV1, MasterEditionV2, Metadata, MAX_EDITION_LEN,
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
    // Deserializing using mpl_token_metadata crate
    if let Ok(metadata) = try_from_slice_checked(&update.data, Key::MetadataV1, MAX_METADATA_LEN) {
        return metadata::process(
            client,
            update.key,
            metadata,
            update.slot,
            update.write_version,
        )
        .await;
    }

    // Deserializing using metaplex_token_metadata and changing the metaplex Metadata to mpl Metadata
    let m: MetaplexMetadata =
        metaplex_try_from_slice_checked(&update.data, MetaplexKey::MetadataV1, MAX_METADATA_LEN)
            .context("failed to deserialize metadata!")?;

    let metaplex_metadata = Metadata {
        key: Key::MetadataV1,
        update_authority: m.update_authority,
        mint: m.mint,
        data: Data {
            name: m.data.name,
            symbol: m.data.symbol,
            uri: m.data.uri,
            seller_fee_basis_points: m.data.seller_fee_basis_points,
            creators: m.data.creators.map(|o| {
                o.into_iter()
                    .map(|c| Creator {
                        address: c.address,
                        verified: c.verified,
                        share: c.share,
                    })
                    .collect()
            }),
        },
        primary_sale_happened: m.primary_sale_happened,
        is_mutable: m.is_mutable,
        edition_nonce: m.edition_nonce,
        token_standard: None,
        collection: None,
        uses: None,
    };

    metadata::process(
        client,
        update.key,
        metaplex_metadata,
        update.slot,
        update.write_version,
    )
    .await
}

async fn process_edition(client: &Client, update: AccountUpdate) -> Result<()> {
    let edition: Edition = try_from_slice_checked(&update.data, Key::EditionV1, MAX_EDITION_LEN)
        .context("Failed to parse edition data")?;

    edition::process(client, update.key, edition, update.slot).await
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

    edition::process_master(client, update.key, master_edition, update.slot).await
}

async fn process_master_edition_v2(client: &Client, update: AccountUpdate) -> Result<()> {
    let master_edition: MasterEditionV2 =
        try_from_slice_checked(&update.data, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
            .context("Failed to parse master edition v2 data")?;

    edition::process_master(client, update.key, master_edition, update.slot).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let first_byte = update.data.first().copied();

    match first_byte {
        None => Ok(()),
        Some(METADATA) => process_metadata(client, update).await,
        Some(EDITION_V1) => process_edition(client, update).await,
        Some(MASTER_EDITION_V1) => process_master_edition_v1(client, update).await,
        Some(MASTER_EDITION_V2) => process_master_edition_v2(client, update).await,
        Some(b) => {
            trace!("Unhandled metadata key byte {:02x}", b);

            Ok(())
        },
    }
}
