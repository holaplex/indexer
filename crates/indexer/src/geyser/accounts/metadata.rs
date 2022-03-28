use indexer_core::{
    db::{
        insert_into,
        models::{Metadata, MetadataCreator},
        tables::{metadata_creators, metadatas},
    },
    pubkeys::find_edition,
};
use metaplex_token_metadata::state::Metadata as MetadataAccount;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, key: Pubkey, meta: MetadataAccount) -> Result<()> {
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
    let first_verified_creator: Option<Pubkey> = meta
        .data
        .creators
        .as_ref()
        .and_then(|creators| creators.iter().find(|c| c.verified).map(|c| c.address));

    client
        .db()
        .run(move |db| {
            insert_into(metadatas::table)
                .values(&row)
                .on_conflict(metadatas::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert metadata")?;

    client
        .dispatch_metadata_json(
            key,
            first_verified_creator,
            meta.data.uri.trim_end_matches('\0').to_owned(),
        )
        .await
        .context("Failed to dispatch metadata JSON job")?;

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
    }

    Ok(())
}
