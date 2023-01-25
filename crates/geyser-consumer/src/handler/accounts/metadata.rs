use indexer::prelude::*;
use indexer_core::{
    db::{
        custom_types::TokenStandardEnum,
        delete, insert_into,
        models::{Metadata, MetadataCollectionKey, MetadataCreator},
        tables::{metadata_collection_keys, metadata_creators, metadatas},
    },
    pubkeys::find_edition,
};
use mpl_token_metadata::state::{Collection, Metadata as MetadataAccount, TokenStandard};

use super::Client;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    meta: MetadataAccount,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let addr = bs58::encode(key).into_string();
    let (edition_pda_key, _bump) = find_edition(meta.mint);
    let metadata = Metadata {
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
        token_standard: meta.token_standard.map(|ts| match ts {
            TokenStandard::NonFungible => TokenStandardEnum::NonFungible,
            TokenStandard::FungibleAsset => TokenStandardEnum::FungibleAsset,
            TokenStandard::Fungible => TokenStandardEnum::Fungible,
            TokenStandard::NonFungibleEdition => TokenStandardEnum::NonFungibleEdition,
        }),
        slot: Some(
            slot.try_into()
                .context("Metadata slot was too big to store")?,
        ),
        burned_at: None,
    };

    let first_verified_creator: Option<Pubkey> = meta
        .data
        .creators
        .as_ref()
        .and_then(|creators| creators.iter().find(|c| c.verified).map(|c| c.address));

    client
        .dispatch_metadata_json(
            key,
            first_verified_creator,
            meta.data.uri.trim_end_matches('\0').to_owned(),
            (slot, write_version),
        )
        .await
        .context("Failed to dispatch metadata JSON job")?;

    client
        .db()
        .run({
            let addr = addr.clone();
            move |db| {
                delete(
                    metadata_creators::table.filter(metadata_creators::metadata_address.eq(addr)),
                )
                .execute(db)
            }
        })
        .await
        .context("Failed to delete metadata creators")?;

    for (position, creator) in meta.data.creators.iter().flatten().enumerate() {
        let row = MetadataCreator {
            metadata_address: Owned(addr.clone()),
            creator_address: Owned(bs58::encode(creator.address).into_string()),
            share: creator.share.into(),
            verified: creator.verified,
            position: Some(
                position
                    .try_into()
                    .context("Position was too big to store")?,
            ),
        };

        client
            .db()
            .run(move |db| {
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

        client
            .db()
            .run({
                let metadata = metadata.clone();
                move |db| {
                    insert_into(metadatas::table)
                        .values(&metadata)
                        .on_conflict(metadatas::address)
                        .do_update()
                        .set(&metadata)
                        .execute(db)
                }
            })
            .await
            .context("Failed to insert metadata")?;
    }

    if meta.collection.is_some() {
        index_metadata_collection_key(client, addr, meta.collection.context("err!")?).await?;
    }

    Ok(())
}

async fn index_metadata_collection_key(
    client: &Client,
    addr: String,
    collection: Collection,
) -> Result<()> {
    let row = MetadataCollectionKey {
        metadata_address: Owned(addr),
        collection_address: Owned(collection.key.to_string()),
        verified: collection.verified,
    };

    client
        .db()
        .run(move |db| {
            insert_into(metadata_collection_keys::table)
                .values(&row)
                .on_conflict(metadata_collection_keys::metadata_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert into metadata_collection_keys")?;

    Ok(())
}
