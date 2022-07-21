use indexer_core::db::{
    sql_query,
    sql_types::{Array, Text},
};
use objects::{nft::Nft, store_creator::StoreCreator};
use scalars::PublicKey;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<StoreCreator>, Vec<Nft>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreCreator>],
    ) -> TryBatchMap<PublicKey<StoreCreator>, Vec<Nft>> {
        let conn = self.db()?;

        let rows: Vec<models::SampleNft> = sql_query(
            "SELECT DISTINCT ON (sample_metadatas.address)
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
                    sample_metadatas.model
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
                    INNER JOIN current_metadata_owners on (metadatas.address = current_metadata_owners.mint_address)
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
