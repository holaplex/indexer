use indexer_core::{
    db::{
        insert_into,
        models::{Metadata, MetadataCreator},
        tables::{metadata_creators, metadatas},
    },
    pubkeys::find_edition,
};
use metaplex_token_metadata::{
    state::{
        Edition as EditionAccount, Key, MasterEditionV2 as MasterEditionAccount,
        Metadata as MetadataAccount, MAX_EDITION_LEN, MAX_MASTER_EDITION_LEN, MAX_METADATA_LEN,
    },
    utils::try_from_slice_checked,
};

use crate::{
    accountsdb::edition::{process_edition, process_master},
    prelude::*,
    Client,
};

const METADATA: u8 = Key::MetadataV1 as u8;
const EDITION_V1: u8 = Key::EditionV1 as u8;
const MASTER_EDITION_V1: u8 = Key::MasterEditionV1 as u8;
const RESERVATION_LIST_V1: u8 = Key::ReservationListV1 as u8;
const RESERVATION_LIST_V2: u8 = Key::ReservationListV2 as u8;
const MASTER_EDITION_V2: u8 = Key::MasterEditionV2 as u8;
const EDITION_MARKER: u8 = Key::EditionMarker as u8;
const UNINITIALIZED_BYTE: u8 = Key::Uninitialized as u8;

async fn process_metadata(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    let meta: MetadataAccount = try_from_slice_checked(&data, Key::MetadataV1, MAX_METADATA_LEN)
        .context("failed to parse metadata")?;

    let addr = bs58::encode(key).into_string();
    let (edition_pda_key, _bump) = find_edition(meta.mint);
    let row = Metadata {
        address: Owned(addr.clone()),
        name: Owned(meta.data.name.trim_end_matches('\0').to_owned()),
        symbol: Owned(meta.data.symbol.trim_end_matches('\0').to_owned()),
        uri: Owned(meta.data.uri.trim_end_matches('\0').to_owned()),
        seller_fee_basis_points: meta.data.seller_fee_basis_points.into(),
        update_authority_address: Owned(bs58::encode(meta.update_authority).into_string()),
        mint_address: Owned(bs58::encode(meta.mint).into_string()),
        primary_sale_happened: meta.primary_sale_happened,
        is_mutable: meta.is_mutable,
        edition_nonce: meta.edition_nonce.map(Into::into),
        edition_pda: Owned(bs58::encode(edition_pda_key).into_string()),
    };

    client
        .db(move |db| {
            insert_into(metadatas::table)
                .values(&row)
                .on_conflict(metadatas::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert metadata")?;

    // TODO
    // handle.push(Job::MetadataUri(
    //     meta_key,
    //     meta.data.uri.trim_end_matches('\0').to_owned(),
    // ));

    for creator in meta.data.creators.unwrap_or_else(Vec::new) {
        let row = MetadataCreator {
            metadata_address: Owned(addr.clone()),
            creator_address: Owned(bs58::encode(creator.address).into_string()),
            share: creator.share.into(),
            verified: creator.verified,
        };

        client
            .db(move |db| {
                insert_into(metadata_creators::table)
                    .values(&row)
                    .on_conflict((
                        metadata_creators::metadata_address,
                        metadata_creators::creator_address,
                    ))
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("Failed to insert metadata creator")?;
    }

    Ok(())
}

fn process_master_edition_v1_data(
    client: &Client,
    master_edition_key: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let master_edition: MasterEditionAccount =
        try_from_slice_checked(&data, Key::MasterEditionV1, MAX_MASTER_EDITION_LEN)
            .context("failed to parse master edition v1 data")?;

    Ok(())
}

fn process_master_edition_v2_data(
    client: &Client,
    master_edition_key: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let master_edition: MasterEditionAccount =
        try_from_slice_checked(&data, Key::MasterEditionV2, MAX_MASTER_EDITION_LEN)
            .context("failed to parse master edition v2 data")?;

    process_master(client, master_edition_key, &master_edition);

    Ok(())
}

fn process_edition_data(client: &Client, meta_key: Pubkey, data: Vec<u8>) -> Result<()> {
    let edition: EditionAccount = try_from_slice_checked(&data, Key::EditionV1, MAX_EDITION_LEN)
        .context("failed to parse edition data")?;

    process_edition(client, meta_key, &edition);

    Ok(())
}

pub(super) async fn process(client: &Client, key: Pubkey, data: Vec<u8>) -> Result<()> {
    let first_byte = data[0] as u8;
    info!("{:?}", first_byte);

    match first_byte {
        _ if first_byte == METADATA => process_metadata(client, key, data).await,
        _ if first_byte == EDITION_V1 => process_edition_data(client, key, data),
        _ if first_byte == MASTER_EDITION_V1 => process_master_edition_v1_data(client, key, data),
        _ if first_byte == MASTER_EDITION_V2 => process_master_edition_v2_data(client, key, data),
        // a if first_byte == EDITION_MARKER => bail!("wagmi EDITION_MARKER {:?}", a),
        a => Ok(()),
    }
}
