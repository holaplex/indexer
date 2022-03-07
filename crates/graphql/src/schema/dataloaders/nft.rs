use objects::nft::{NftAttribute, NftCreator, NftOwner};
use scalars::PublicKey;
use tables::{attributes, metadata_creators, token_accounts};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey, Vec<NftAttribute>> for Batcher {
    async fn load(&mut self, addresses: &[PublicKey]) -> TryBatchMap<PublicKey, Vec<NftAttribute>> {
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
impl TryBatchFn<PublicKey, Vec<NftCreator>> for Batcher {
    async fn load(&mut self, addresses: &[PublicKey]) -> TryBatchMap<PublicKey, Vec<NftCreator>> {
        let conn = self.db()?;

        let rows: Vec<models::MetadataCreator> = metadata_creators::table
            .filter(metadata_creators::metadata_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load NFT creators")?;

        Ok(rows
            .into_iter()
            .map(|c| (c.metadata_address.clone(), c.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey, Option<NftOwner>> for Batcher {
    async fn load(
        &mut self,
        mint_addresses: &[PublicKey],
    ) -> TryBatchMap<PublicKey, Option<NftOwner>> {
        let conn = self.db()?;

        let rows: Vec<models::TokenAccount> = token_accounts::table
            .filter(token_accounts::mint_address.eq(any(mint_addresses)))
            .filter(token_accounts::amount.eq(1))
            .load(&conn)
            .context("Failed to load NFT owners")?;

        Ok(rows
            .into_iter()
            .map(|t| {
                (t.mint_address.into_owned(), NftOwner {
                    address: t.owner_address.into_owned(),
                })
            })
            .batch(mint_addresses))
    }
}
