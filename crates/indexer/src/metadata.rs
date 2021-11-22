use indexer_core::db::{
    insert_into,
    models::{Metadata, MetadataCreator},
    tables::{
        metadata_creators::{creator_address, metadata_address, metadata_creators},
        metadatas::{address, metadatas},
    },
};
use metaplex_token_metadata::state::Metadata as MetadataAccount;
use solana_sdk::pubkey::Pubkey;

use crate::{prelude::*, util, Client, Job, ThreadPoolHandle};

pub fn process(client: &Client, meta_key: Pubkey, handle: ThreadPoolHandle) -> Result<()> {
    let mut acct = client
        .get_account(&meta_key)
        .context("Failed to get item metadata")?;

    let meta = MetadataAccount::from_account_info(&util::account_as_info(
        &meta_key, false, false, &mut acct,
    ))
    .context("Failed tintegeio parse Metadata")?;

    let addr = meta_key.to_bytes();
    let auth_addr = meta.update_authority.to_bytes();
    let mint_addr = meta.mint.to_bytes();
    let row = Metadata {
        address: Borrowed(&addr),
        name: Borrowed(meta.data.name.trim_end_matches('\0')),
        symbol: Borrowed(meta.data.symbol.trim_end_matches('\0')),
        uri: Borrowed(meta.data.uri.trim_end_matches('\0')),
        seller_fee_basis_points: meta.data.seller_fee_basis_points.into(),
        update_authority_address: Borrowed(&auth_addr),
        mint_address: Borrowed(&mint_addr),
        primary_sale_happened: meta.primary_sale_happened,
        is_mutable: meta.is_mutable,
        edition_nonce: meta.edition_nonce.map(Into::into),
    };

    let db = client.db()?;

    insert_into(metadatas)
        .values(&row)
        .on_conflict(address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert metadata")?;

    handle.push(Job::EditionForMint(meta.mint));

    for creator in meta.data.creators.unwrap_or_else(Vec::new) {
        let creator_addr = creator.address.to_bytes();
        let row = MetadataCreator {
            metadata_address: Borrowed(&addr),
            creator_address: Borrowed(&creator_addr),
            share: creator.share.into(),
            verified: creator.verified,
        };

        insert_into(metadata_creators)
            .values(&row)
            .on_conflict((metadata_address, creator_address))
            .do_update()
            .set(&row)
            .execute(&db)
            .context("Failed to insert metadata creator")?;
    }

    Ok(())
}
