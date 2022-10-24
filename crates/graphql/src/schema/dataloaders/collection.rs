use indexer_core::db::{
    sql_query,
    sql_types::{Array, Text},
    tables::{collection_trends, collections},
};
use objects::{
    collections::{Collection, CollectionId},
    nft::{CollectionNFT, Nft},
    store_creator::StoreCreator,
};
use scalars::{PublicKey, I64};

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

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CollectionHoldersCount(pub I64);

impl From<i64> for CollectionHoldersCount {
    fn from(value: i64) -> Self {
        Self(value.into())
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CollectionNFT>, Option<CollectionNftCount>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CollectionNFT>],
    ) -> TryBatchMap<PublicKey<CollectionNFT>, Option<CollectionNftCount>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionCount> = sql_query(
            "SELECT COLLECTION_ID::text AS COLLECTION, NFT_COUNT as COUNT
            FROM ME_COLLECTION_STATS
            where COLLECTION_ID::text = ANY($1)
            UNION ALL
            SELECT COLLECTION_ADDRESS AS COLLECTION, NFT_COUNT as COUNT
            FROM COLLECTION_STATS
            where COLLECTION_ADDRESS = ANY($1);
            -- $1: addresses::text[]",
        )
        .bind::<Array<Text>, _>(addresses)
        .load(&conn)
        .context("Failed to load NFT count for collection")?;

        Ok(rows
            .into_iter()
            .map(|models::CollectionCount { collection, count }| {
                (collection, CollectionNftCount::from(count))
            })
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CollectionNFT>, Option<CollectionHoldersCount>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CollectionNFT>],
    ) -> TryBatchMap<PublicKey<CollectionNFT>, Option<CollectionHoldersCount>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionCount> = sql_query(
            "SELECT COLLECTION, HOLDERS_COUNT as COUNT
            FROM
                (
                    (
                    SELECT  DISTINCT COUNT(*) OVER () AS holders_count, METADATA_COLLECTION_KEYS.COLLECTION_ADDRESS as collection
                    FROM METADATA_COLLECTION_KEYS
                    INNER JOIN METADATAS ON METADATAS.ADDRESS = METADATA_COLLECTION_KEYS.METADATA_ADDRESS
                    INNER JOIN CURRENT_METADATA_OWNERS ON CURRENT_METADATA_OWNERS.MINT_ADDRESS = METADATAS.MINT_ADDRESS
                    WHERE METADATAS.BURNED_AT IS NULL
                        AND METADATA_COLLECTION_KEYS.COLLECTION_ADDRESS = ANY($1)
                    GROUP BY (METADATA_COLLECTION_KEYS.COLLECTION_ADDRESS, CURRENT_METADATA_OWNERS.OWNER_ADDRESS))
            UNION
                    (
                    SELECT DISTINCT COUNT(*) OVER () AS holders_count, ME_METADATA_COLLECTIONS.COLLECTION_ID::text as collection
                    FROM ME_METADATA_COLLECTIONS
                    INNER JOIN METADATAS ON METADATAS.ADDRESS = ME_METADATA_COLLECTIONS.METADATA_ADDRESS
                    INNER JOIN CURRENT_METADATA_OWNERS ON CURRENT_METADATA_OWNERS.MINT_ADDRESS = METADATAS.MINT_ADDRESS
                    WHERE METADATAS.BURNED_AT IS NULL
                        AND ME_METADATA_COLLECTIONS.COLLECTION_ID::text = ANY($1)
                    GROUP BY (ME_METADATA_COLLECTIONS.COLLECTION_ID::text, CURRENT_METADATA_OWNERS.OWNER_ADDRESS))) AS A;
            -- $1: addresses::text[]",
        )
        .bind::<Array<Text>, _>(addresses)
        .load(&conn)
        .context("Failed to load holder count for collection")?;

        Ok(rows
            .into_iter()
            .map(|models::CollectionCount { collection, count }| {
                (collection, CollectionHoldersCount::from(count))
            })
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<CollectionNFT>, Option<CollectionFloorPrice>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<CollectionNFT>],
    ) -> TryBatchMap<PublicKey<CollectionNFT>, Option<CollectionFloorPrice>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionFloorPrice> = sql_query(
            "SELECT COLLECTION_ID::text AS COLLECTION, FLOOR_PRICE
            FROM ME_COLLECTION_STATS
            where COLLECTION_ID::text = ANY($1)
            UNION ALL
            SELECT COLLECTION_ADDRESS AS COLLECTION, FLOOR_PRICE
            FROM COLLECTION_STATS
            where COLLECTION_ADDRESS = ANY($1);",
        )
        .bind::<Array<Text>, _>(addresses)
        .load(&conn)
        .context("Failed to load floor price for collection")?;

        Ok(rows
            .into_iter()
            .filter_map(
                |models::CollectionFloorPrice {
                     collection,
                     floor_price,
                 }| floor_price.map(|f| (collection, f)),
            )
            .map(|(collection, f)| (collection, CollectionFloorPrice::from(f)))
            .batch(addresses))
    }
}

// Moon rank dataloader
#[async_trait]
impl TryBatchFn<CollectionId, Option<Collection>> for Batcher {
    async fn load(
        &mut self,
        identifiers: &[String],
    ) -> TryBatchMap<CollectionId, Option<Collection>> {
        let conn = self.db()?;

        let rows: Vec<models::Collection> = collections::table
            .select(collections::all_columns)
            .filter(collections::id.eq(any(identifiers)))
            .load(&conn)
            .context("failed to load mr collection")?;

        Ok(rows
            .into_iter()
            .map(|a| (a.id.clone(), a.try_into()))
            .batch(identifiers))
    }
}

#[async_trait]
impl TryBatchFn<CollectionId, Option<CollectionNftCount>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[CollectionId],
    ) -> TryBatchMap<CollectionId, Option<CollectionNftCount>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionCount> = sql_query(
            "SELECT COLLECTION_ID AS COLLECTION, COUNT(*) as COUNT
            FROM COLLECTION_MINTS
            GROUP BY COLLECTION_ID
            where COLLECTION_ADDRESS = ANY($1);
            -- $1: addresses::text[]",
        )
        .bind::<Array<Text>, _>(addresses)
        .load(&conn)
        .context("Failed to load NFT count for mr collection")?;

        Ok(rows
            .into_iter()
            .map(|models::CollectionCount { collection, count }| {
                (collection, CollectionNftCount::from(count))
            })
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<CollectionId, Option<CollectionHoldersCount>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[CollectionId],
    ) -> TryBatchMap<CollectionId, Option<CollectionHoldersCount>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionCount> = sql_query(
            "SELECT  DISTINCT COUNT(*) OVER () AS holders_count, COLLECTION_MINTS.COLLECTION_ID as collection
            FROM COLLECTION_MINTS
            INNER JOIN METADATAS ON METADATAS.MINT_ADDRESS = COLLECTION_MINTS.MINT
            INNER JOIN CURRENT_METADATA_OWNERS ON CURRENT_METADATA_OWNERS.MINT_ADDRESS = METADATAS.MINT_ADDRESS
            WHERE METADATAS.BURNED_AT IS NULL
                AND COLLECTION_MINTS.COLLECTION_ID = ANY($1)
            GROUP BY (COLLECTION_MINTS.COLLECTION_ID, CURRENT_METADATA_OWNERS.OWNER_ADDRESS));
            -- $1: addresses::text[]",
        )
        .bind::<Array<Text>, _>(addresses)
        .load(&conn)
        .context("Failed to load holder count for mr collection")?;

        Ok(rows
            .into_iter()
            .map(|models::CollectionCount { collection, count }| {
                (collection, CollectionHoldersCount::from(count))
            })
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<CollectionId, Option<objects::nft::CollectionTrend>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[CollectionId],
    ) -> TryBatchMap<CollectionId, Option<objects::nft::CollectionTrend>> {
        let conn = self.db()?;

        let rows: Vec<models::CollectionTrend> = collection_trends::table
            .select(collection_trends::all_columns)
            .filter(collection_trends::collection.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load collection trends")?;

        Ok(rows
            .into_iter()
            .map(|a| (a.collection.clone(), a.try_into()))
            .batch(addresses))
    }
}
