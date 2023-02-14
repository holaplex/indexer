use borsh::{BorshDeserialize, BorshSerialize};
use indexer::prelude::*;
use mpl_token_metadata::{
    state::{
        Collection, Data, Edition, Key, MasterEditionV1, MasterEditionV2, Metadata, Uses,
        MAX_EDITION_LEN, MAX_MASTER_EDITION_LEN,
    },
    utils::try_from_slice_checked,
};

use super::{
    accounts::{edition, metadata},
    AccountUpdate, Client,
};

const METADATA: u8 = Key::MetadataV1 as u8;
const EDITION_V1: u8 = Key::EditionV1 as u8;
const MASTER_EDITION_V1: u8 = Key::MasterEditionV1 as u8;
const MASTER_EDITION_V2: u8 = Key::MasterEditionV2 as u8;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]

pub struct ProgrammableConfigStruct(Option<ProgrammableConfig>);
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]

pub enum ProgrammableConfig {
    V1 { rule_set: Option<Pubkey> },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]

pub enum CollectionDetails {
    V1 { size: u64 },
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]

pub struct TokenStandardStruct(pub Option<TokenStandard>);

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]

pub enum TokenStandard {
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
}

async fn process_metadata(client: &Client, update: AccountUpdate) -> Result<()> {
    let buf = &mut update.data.as_slice();
    let (metadata, programmable_config, token_standard) =
        metadata_deser(buf).context("failed to deserialize metadata")?;

    metadata::process(
        client,
        update.key,
        metadata,
        programmable_config,
        token_standard,
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

/// https://docs.rs/mpl-token-metadata/1.8.3/src/mpl_token_metadata/utils/metadata.rs.html#192
fn metadata_deser(
    buf: &mut &[u8],
) -> Result<(Metadata, Option<ProgrammableConfig>, Option<TokenStandard>)> {
    let key: Key = BorshDeserialize::deserialize(buf)?;
    let update_authority: Pubkey = BorshDeserialize::deserialize(buf)?;
    let mint: Pubkey = BorshDeserialize::deserialize(buf)?;
    let data: Data = BorshDeserialize::deserialize(buf)?;
    let primary_sale_happened: bool = BorshDeserialize::deserialize(buf)?;
    let is_mutable: bool = BorshDeserialize::deserialize(buf)?;
    let edition_nonce: Option<u8> = BorshDeserialize::deserialize(buf)?;

    let token_standard_res: Result<Option<TokenStandard>> =
        BorshDeserialize::deserialize(buf).map_err(Into::into);
    let collection_res: Result<Option<Collection>> =
        BorshDeserialize::deserialize(buf).map_err(Into::into);
    let uses_res: Result<Option<Uses>> = BorshDeserialize::deserialize(buf).map_err(Into::into);

    // V1.3
    let _collection_details_res: Result<Option<CollectionDetails>> =
        BorshDeserialize::deserialize(buf).map_err(Into::into);

    // pNFT - Programmable Config
    let programmable_config_res: Result<Option<ProgrammableConfig>> =
        BorshDeserialize::deserialize(buf).map_err(Into::into);

    let (token_standard, collection, uses) = match (token_standard_res, collection_res, uses_res) {
        (Ok(token_standard_res), Ok(collection_res), Ok(uses_res)) => {
            (token_standard_res, collection_res, uses_res)
        },
        _ => (None, None, None),
    };

    // Programmable Config
    let programmable_config = programmable_config_res.unwrap_or(None);

    Ok((
        Metadata {
            key,
            update_authority,
            mint,
            data,
            primary_sale_happened,
            is_mutable,
            edition_nonce,
            token_standard: None,
            collection,
            uses,
        },
        programmable_config,
        token_standard,
    ))
}
