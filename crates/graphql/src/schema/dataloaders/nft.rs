use objects::nft::{NftAttribute, NftCreator, NftOwner};
use tables::{attributes, metadata_creators, token_accounts};

use super::prelude::*;

pub struct NftAttributeBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<NftAttribute>> for NftAttributeBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<NftAttribute>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for address in addresses {
            hash_map.insert(address.clone(), Vec::new());
        }

        let rows: Vec<models::MetadataAttribute> = attributes::table
            .filter(attributes::metadata_address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, attribute: models::MetadataAttribute| {
                let attribute = NftAttribute::try_from(attribute).unwrap();
                acc.entry(attribute.metadata_address.clone())
                    .and_modify(|attributes| {
                        attributes.push(attribute);
                    });
                acc
            })
    }
}

pub struct NftCreatorBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<NftCreator>> for NftCreatorBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<NftCreator>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for creator in addresses {
            hash_map.insert(creator.clone(), Vec::new());
        }

        let rows: Vec<models::MetadataCreator> = metadata_creators::table
            .filter(metadata_creators::metadata_address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, creator: models::MetadataCreator| {
                let creator = NftCreator::from(creator);
                acc.entry(creator.metadata_address.clone())
                    .and_modify(|creators| {
                        creators.push(creator);
                    });
                acc
            })
    }
}

pub struct NftOwnerBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<NftOwner>> for NftOwnerBatcher {
    async fn load(&mut self, mint_addresses: &[String]) -> HashMap<String, Option<NftOwner>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for address in mint_addresses {
            hash_map.insert(address.clone(), None);
        }

        let token_accounts: Vec<models::TokenAccount> = token_accounts::table
            .filter(token_accounts::mint_address.eq(any(mint_addresses)))
            .filter(token_accounts::amount.eq(1))
            .load(&conn)
            .unwrap();

        token_accounts.into_iter().fold(hash_map, |mut acc, ta| {
            acc.insert(
                ta.mint_address.into_owned(),
                Some(NftOwner {
                    address: ta.owner_address.into_owned(),
                }),
            );
            acc
        })
    }
}
