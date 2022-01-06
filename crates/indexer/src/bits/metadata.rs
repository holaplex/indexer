use indexer_core::db::{
    insert_into,
    models::{Metadata, MetadataCreator},
    tables::{metadata_creators, metadatas},
};
use metaplex_token_metadata::state::Metadata as MetadataAccount;

use crate::{prelude::*, util, Client, EditionKeys, Job, ThreadPoolHandle};

pub fn process(client: &Client, meta_key: Pubkey, handle: ThreadPoolHandle) -> Result<()> {
    let mut acct = client
        .get_account(&meta_key)
        .context("Failed to get item metadata")?;

    let meta = MetadataAccount::from_account_info(&util::account_as_info(
        &meta_key, false, false, &mut acct,
    ))
    .context("Failed tintegeio parse Metadata")?;

    let addr = bs58::encode(meta_key).into_string();
    let row = Metadata {
        address: Borrowed(&addr),
        name: Borrowed(meta.data.name.trim_end_matches('\0')),
        symbol: Borrowed(meta.data.symbol.trim_end_matches('\0')),
        uri: Borrowed(meta.data.uri.trim_end_matches('\0')),
        seller_fee_basis_points: meta.data.seller_fee_basis_points.into(),
        update_authority_address: Owned(bs58::encode(meta.update_authority).into_string()),
        mint_address: Owned(bs58::encode(meta.mint).into_string()),
        primary_sale_happened: meta.primary_sale_happened,
        is_mutable: meta.is_mutable,
        edition_nonce: meta.edition_nonce.map(Into::into),
    };

    let db = client.db()?;

    insert_into(metadatas::table)
        .values(&row)
        .on_conflict(metadatas::address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert metadata")?;

    handle.push(Job::EditionForMint(EditionKeys {
        mint: meta.mint,
        metadata: meta_key,
    }));

    for creator in meta.data.creators.unwrap_or_else(Vec::new) {
        let row = MetadataCreator {
            metadata_address: Borrowed(&addr),
            creator_address: Owned(bs58::encode(creator.address).into_string()),
            share: creator.share.into(),
            verified: creator.verified,
        };

        insert_into(metadata_creators::table)
            .values(&row)
            .on_conflict((
                metadata_creators::metadata_address,
                metadata_creators::creator_address,
            ))
            .do_update()
            .set(&row)
            .execute(&db)
            .context("Failed to insert metadata creator")?;
    }

    Ok(())
}
