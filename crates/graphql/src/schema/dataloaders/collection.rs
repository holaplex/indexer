use indexer_core::db::{
    sql_query,
    sql_types::{Array, Text},
};
use objects::{
    nft::{Collection, Nft},
    store_creator::StoreCreator,
};
use scalars::{PublicKey, I64};
use tables::collection_stats;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<StoreCreator>, Vec<Nft>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreCreator>],
    ) -> TryBatchMap<PublicKey<StoreCreator>, Vec<Nft>> {
        let conn = self.db()?;

        let rows: Vec<models::SampleNft> = sql_query(
            "SELECT sample_metadatas.address,
                    sample_metadatas.creator_address,
                    sample_metadatas.address,
                    sample_metadatas.name,
                    sample_metadatas.seller_fee_basis_points,
                    sample_metadatas.mint_address,
                    sample_metadatas.primary_sale_happened,
                    sample_metadatas.update_authority_address,
                    sample_metadatas.uri,
                    sample_metadatas.description,
                    sample_metadatas.image,
                    sample_metadatas.animation_url,
                    sample_metadatas.external_url,
                    sample_metadatas.category,
                    sample_metadatas.model,
                    sample_metadatas.token_account_address
                FROM store_creators
                JOIN LATERAL (
                    SELECT
                        metadatas.address AS address,
                        metadatas.name AS name,
                        metadatas.seller_fee_basis_points AS seller_fee_basis_points,
                        metadatas.mint_address AS mint_address,
                        metadatas.primary_sale_happened AS primary_sale_happened,
                        metadatas.update_authority_address AS update_authority_address,
                        current_metadata_owners.token_account_address AS token_account_address,
                        metadatas.uri AS uri,
                        metadata_jsons.description AS description,
                        metadata_jsons.image AS image,
                        metadata_jsons.animation_url AS animation_url,
                        metadata_jsons.external_url AS external_url,
                        metadata_jsons.category AS category,
                        metadata_jsons.model AS model,
                        store_creators.creator_address AS creator_address
                    FROM metadatas
                    INNER JOIN metadata_jsons ON (metadatas.address = metadata_jsons.metadata_address)
                    INNER JOIN metadata_creators ON (metadatas.address = metadata_creators.metadata_address)
                    INNER JOIN current_metadata_owners on (metadatas.mint_address = current_metadata_owners.mint_address)
                    WHERE metadata_creators.creator_address = store_creators.creator_address
                    LIMIT 3
                ) AS sample_metadatas ON true
                WHERE store_creators.creator_address = ANY($1);",
        ).bind::<Array<Text>, _>(addresses)
            .load(&conn)
            .context("Failed to load collection preview(s)")?;

        Ok(rows
            .into_iter()
            .map(
                |models::SampleNft {
                     creator_address,
                     address,
                     name,
                     seller_fee_basis_points,
                     mint_address,
                     token_account_address,
                     primary_sale_happened,
                     update_authority_address,
                     uri,
                     description,
                     image,
                     animation_url,
                     external_url,
                     category,
                     model,
                 }| {
                    (
                        creator_address,
                        models::Nft {
                            address,
                            name,
                            seller_fee_basis_points,
                            mint_address,
                            token_account_address,
                            primary_sale_happened,
                            update_authority_address,
                            uri,
                            description,
                            image,
                            animation_url,
                            external_url,
                            category,
                            model,
                            slot: None,
                        }
                        .try_into(),
                    )
                },
            )
            .batch(addresses))
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CollectionNftCount(pub I64);

impl From<i64> for CollectionNftCount {
    fn from(value: i64) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CollectionFloorPrice(pub I64);

impl From<i64> for CollectionFloorPrice {
    fn from(value: i64) -> Self {
        Self(value.into())
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Collection>, Option<CollectionNftCount>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Collection>],
    ) -> TryBatchMap<PublicKey<Collection>, Option<CollectionNftCount>> {
        let conn = self.db()?;

        let rows: Vec<(String, i64)> = collection_stats::table
            .filter(collection_stats::collection_address.eq(any(addresses)))
            .select((
                collection_stats::collection_address,
                collection_stats::nft_count,
            ))
            .load(&conn)
            .context("Failed to load NFT count for collection")?;

        Ok(rows
            .into_iter()
            .map(|(addr, count)| (addr, CollectionNftCount::from(count)))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Collection>, Option<CollectionFloorPrice>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Collection>],
    ) -> TryBatchMap<PublicKey<Collection>, Option<CollectionFloorPrice>> {
        let conn = self.db()?;

        let rows: Vec<(String, Option<i64>)> = collection_stats::table
            .filter(collection_stats::collection_address.eq(any(addresses)))
            .select((
                collection_stats::collection_address,
                collection_stats::floor_price,
            ))
            .load(&conn)
            .context("Failed to load floor price for collection")?;

        Ok(rows
            .into_iter()
            .filter_map(|(addr, floor)| floor.map(|f| (addr, f)))
            .map(|(addr, floor)| (addr, CollectionFloorPrice::from(floor)))
            .batch(addresses))
    }
}
