use indexer_core::db::queries;
use objects::{
    listing_receipt::ListingReceipt,
    nft::{Nft, NftActivity, NftAttribute, NftCreator, NftOwner},
    purchase_receipt::PurchaseReceipt,
};
use scalars::PublicKey;
use tables::{
    attributes, listing_receipts, metadata_creators, metadatas, purchase_receipts, token_accounts,
    twitter_handle_name_services,
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

        let rows: Vec<(Option<String>, models::TokenAccount)> =
            token_accounts::table
                .left_join(twitter_handle_name_services::table.on(
                    twitter_handle_name_services::wallet_address.eq(token_accounts::owner_address),
                ))
                .filter(token_accounts::mint_address.eq(any(mint_addresses)))
                .filter(token_accounts::amount.eq(1))
                .select((
                    twitter_handle_name_services::twitter_handle.nullable(),
                    (
                        token_accounts::address,
                        token_accounts::mint_address,
                        token_accounts::owner_address,
                        token_accounts::amount,
                        token_accounts::slot,
                    ),
                ))
                .load(&conn)
                .context("Failed to load NFT owners")?;

        Ok(rows
            .into_iter()
            .map(|(h, t)| {
                (t.mint_address.into_owned(), NftOwner {
                    address: t.owner_address.into_owned(),
                    associated_token_account_address: t.address.into_owned(),
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
                token_accounts::table.on(token_accounts::mint_address.eq(metadatas::mint_address)),
            )
            .select(listing_receipts::all_columns)
            .filter(token_accounts::amount.eq(1))
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
