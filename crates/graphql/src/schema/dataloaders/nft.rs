use indexer_core::db::{queries, tables::metadata_collection_keys};
use objects::{
    listing_receipt::ListingReceipt,
    nft::{Collection, Nft, NftActivity, NftAttribute, NftCreator, NftFile, NftOwner},
    purchase_receipt::PurchaseReceipt,
};
use scalars::PublicKey;
use tables::{
    attributes, current_metadata_owners, files, listing_receipts, metadata_creators,
    metadata_jsons, metadatas, purchase_receipts, twitter_handle_name_services,
};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<NftAttribute>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<NftAttribute>> {
        let conn = self.db()?;

        let rows: Vec<models::MetadataAttribute> = attributes::table
            .filter(attributes::metadata_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load NFT attributes")?;

        Ok(rows
            .into_iter()
            .map(|a| (a.metadata_address.clone(), a.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Option<Collection>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Option<Collection>> {
        let conn = self.db()?;

        let rows: Vec<(String, models::Nft)> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .inner_join(
                metadata_collection_keys::table
                    .on(metadata_collection_keys::collection_address.eq(metadatas::mint_address)),
            )
            .filter(metadata_collection_keys::metadata_address.eq(any(addresses)))
            .select((
                metadata_collection_keys::metadata_address,
                queries::metadatas::NFT_COLUMNS,
            ))
            .load(&conn)
            .context("Failed to load Collection NFTs")?;

        Ok(rows
            .into_iter()
            .map(|(addr, nft)| (addr, nft.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<NftCreator>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<NftCreator>> {
        let conn = self.db()?;

        let rows: Vec<(Option<String>, models::MetadataCreator)> = metadata_creators::table
            .left_join(twitter_handle_name_services::table.on(
                twitter_handle_name_services::wallet_address.eq(metadata_creators::creator_address),
            ))
            .filter(metadata_creators::metadata_address.eq(any(addresses)))
            .order(metadata_creators::position.asc())
            .select((
                twitter_handle_name_services::twitter_handle.nullable(),
                (metadata_creators::all_columns),
            ))
            .load(&conn)
            .context("Failed to load NFT creators")?;

        Ok(rows
            .into_iter()
            .map(|c| (c.1.metadata_address.clone(), c.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Option<NftOwner>> for Batcher {
    async fn load(
        &mut self,
        mint_addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Option<NftOwner>> {
        let conn = self.db()?;

        let rows: Vec<(Option<String>, models::CurrentMetadataOwner)> =
            current_metadata_owners::table
                .left_join(
                    twitter_handle_name_services::table
                        .on(twitter_handle_name_services::wallet_address
                            .eq(current_metadata_owners::owner_address)),
                )
                .filter(current_metadata_owners::mint_address.eq(any(mint_addresses)))
                .select((
                    twitter_handle_name_services::twitter_handle.nullable(),
                    (
                        current_metadata_owners::mint_address,
                        current_metadata_owners::owner_address,
                        current_metadata_owners::token_account_address,
                        current_metadata_owners::slot,
                    ),
                ))
                .load(&conn)
                .context("Failed to load NFT owners")?;

        Ok(rows
            .into_iter()
            .map(|(h, t)| {
                (t.mint_address.into_owned(), NftOwner {
                    address: t.owner_address.into_owned(),
                    associated_token_account_address: t.token_account_address.into_owned(),
                    twitter_handle: h,
                })
            })
            .batch(mint_addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<PurchaseReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<PurchaseReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::PurchaseReceipt> = purchase_receipts::table
            .inner_join(metadatas::table.on(metadatas::address.eq(purchase_receipts::metadata)))
            .select(purchase_receipts::all_columns)
            .filter(purchase_receipts::metadata.eq(any(addresses)))
            .order(purchase_receipts::created_at.desc())
            .load(&conn)
            .context("Failed to load purchase receipts")?;

        Ok(rows
            .into_iter()
            .map(|purchase| (purchase.metadata.clone(), purchase.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<ListingReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<ListingReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::ListingReceipt> = listing_receipts::table
            .inner_join(metadatas::table.on(metadatas::address.eq(listing_receipts::metadata)))
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .select(listing_receipts::all_columns)
            .filter(listing_receipts::canceled_at.is_null())
            .filter(listing_receipts::purchase_receipt.is_null())
            .filter(listing_receipts::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load listing receipts")?;

        Ok(rows
            .into_iter()
            .map(|listing| (listing.metadata.clone(), listing.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<NftActivity>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<NftActivity>> {
        let conn = self.db()?;

        let rows = queries::metadatas::activities(&conn, addresses)?;

        Ok(rows
            .into_iter()
            .map(|activity| (activity.metadata.clone(), activity.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<NftFile>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<NftFile>> {
        let conn = self.db()?;

        let rows: Vec<models::MetadataFile> = files::table
            .filter(files::metadata_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load NFT files")?;

        Ok(rows
            .into_iter()
            .map(|a| (a.metadata_address.clone(), a.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Option<Nft>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Option<Nft>> {
        let conn = self.db()?;

        let rows: Vec<models::Nft> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .inner_join(
                current_metadata_owners::table
                    .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
            )
            .filter(metadatas::address.eq(any(addresses)))
            .select(queries::metadatas::NFT_COLUMNS)
            .load(&conn)
            .context("Failed to load NFTs")?;

        Ok(rows
            .into_iter()
            .map(|nft| (nft.address.clone(), nft.try_into()))
            .batch(addresses))
    }
}
