use indexer_core::{
    db::{
        insert_into,
        models::{Metadata, MetadataCreator},
        tables::{metadata_creators, metadatas},
    },
    pubkeys,
};
use metaplex_token_metadata::state::Metadata as MetadataAccount;

use crate::{client::prelude::*, prelude::*, util, Client, EditionKeys, Job, ThreadPoolHandle};

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;
pub const FIRST_CREATOR_LENGTH: usize = 1
    + 32
    + 32
    + 4
    + MAX_NAME_LENGTH
    + 4
    + MAX_URI_LENGTH
    + 4
    + MAX_SYMBOL_LENGTH
    + 2
    + 1
    + 4
    + 0 * MAX_CREATOR_LEN;

fn get_metadatas_by_primary_creator(
    client: &Client,
    creator_address: Pubkey,
) -> Result<
    Vec<(solana_sdk::pubkey::Pubkey, solana_sdk::account::Account)>,
    solana_client::client_error::ClientError,
> {
    client.get_program_accounts(pubkeys::metadata(), RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: FIRST_CREATOR_LENGTH,
            bytes: MemcmpEncodedBytes::Base58(creator_address.to_string()),
            encoding: None,
        })]),
        ..RpcProgramAccountsConfig::default()
    })
}

pub fn get_metadata_by_creator(
    client: &Client,
    pubkey: Pubkey,
    handle: ThreadPoolHandle,
) -> Result<()> {
    let metadatas = get_metadatas_by_primary_creator(client, pubkey)
        .context("failed to get metadatas by creator")?;

    for metadata in metadatas {
        handle.push(Job::Metadata(metadata.0));
    }

    Ok(())
}

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
