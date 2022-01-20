use std::collections::HashMap;

use indexer_core::{
    db::{
        models,
        tables::{metadata_creators, metadatas},
        Pool,
    },
    prelude::*,
};
use juniper::{EmptySubscription, FieldResult, GraphQLInputObject, GraphQLObject, RootNode};
#[derive(GraphQLObject)]
#[graphql(description = "A Solana NFT")]
struct Nft {
    address: String,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: i32,
    update_authority_address: String,
    mint_address: String,
    primary_sale_happened: bool,
    is_mutable: bool,
    creators: Vec<String>,
}

// impl<'a> From<models::Metadata<'a>> for Nft {
//     fn from(
//         models::Metadata {
//             address,
//             name,
//             symbol,
//             uri,
//             seller_fee_basis_points,
//             update_authority_address,
//             mint_address,
//             primary_sale_happened,
//             is_mutable,
//             edition_nonce: _,
//         }: models::Metadata,
//     ) -> Self {
//         Self {
//             address: address.into_owned(),
//             name: name.into_owned(),
//             uri: uri.into_owned(),
//             symbol: symbol.into_owned(),
//             seller_fee_basis_points,
//             update_authority_address: update_authority_address.into_owned(),
//             mint_address: mint_address.into_owned(),
//             primary_sale_happened,
//             is_mutable,
//         }
//     }
// }

#[derive(GraphQLInputObject)]
#[graphql(description = "Buy a NFT")]
struct BuyNft {
    transaction: String,
}

pub struct QueryRoot {
    db: Pool,
}

#[juniper::graphql_object]
impl QueryRoot {
    fn nfts(
        &self,
        #[graphql(description = "Filter on creator address")] creators: Option<Vec<String>>,
        #[graphql(description = "Filter on update authority addres")] update_authority: Option<
            Vec<String>,
        >,
    ) -> Vec<Nft> {
        let conn = self.db.get().unwrap();

        // Create a hashmap for all Nfts found
        let mut nfts_hash: HashMap<String, Nft> = HashMap::new();

        // Create mutable vector for all rows returned
        // let mut all_rows: Vec<String> = Vec::new();

        // for every creator in creators parameter
        for creator in creators.into_iter().flatten().collect::<Vec<String>>() {
            // get all token addresses by creator
            let tokens: Vec<String> = metadata_creators::table
                .select(metadata_creators::metadata_address)
                .filter(metadata_creators::creator_address.eq(&creator))
                .load(&conn)
                .unwrap();

            // get nft from token address
            for address in &tokens {
                // get the token metadata
                let token: Vec<models::Metadata> = metadatas::table
                    .select(metadatas::all_columns)
                    .filter(metadatas::address.eq(address))
                    .limit(1)
                    .load(&conn)
                    .unwrap();

                // get the nft from the hashmap or create a new object
                nfts_hash.entry(address.clone()).or_insert(Nft {
                    address: token[0].address.to_string(),
                    name: token[0].name.to_string(),
                    symbol: token[0].symbol.to_string(),
                    uri: token[0].uri.to_string(),
                    seller_fee_basis_points: token[0].seller_fee_basis_points,
                    update_authority_address: "".to_owned(),
                    mint_address: token[0].mint_address.to_string(),
                    primary_sale_happened: token[0].primary_sale_happened,
                    is_mutable: token[0].is_mutable,
                    creators: vec![]
                });
            }
        }

        // for every update authority in ua parameter
        for ua in update_authority.into_iter().flatten().collect::<Vec<String>>() {
            // get all token addresses by creator
            let tokens: Vec<String> = metadata_creators::table
                .select(metadata_creators::metadata_address)
                .filter(metadata_creators::creator_address.eq(&ua))
                .load(&conn)
                .unwrap();

            // get nft from token address
            for address in &tokens {
                // get the token metadata
                let token: Vec<models::Metadata> = metadatas::table
                    .select(metadatas::all_columns)
                    .filter(metadatas::address.eq(address))
                    .limit(1)
                    .load(&conn)
                    .unwrap();

                // get the nft from the hashmap or create a new object
                nfts_hash.entry(address.clone()).or_insert(Nft {
                    address: token[0].address.to_string(),
                    name: token[0].name.to_string(),
                    symbol: token[0].symbol.to_string(),
                    uri: token[0].uri.to_string(),
                    seller_fee_basis_points: token[0].seller_fee_basis_points,
                    update_authority_address: ua.to_owned(),
                    mint_address: token[0].mint_address.to_string(),
                    primary_sale_happened: token[0].primary_sale_happened,
                    is_mutable: token[0].is_mutable,
                    creators: vec![]
                });
            }
        }
        
        let mut all_nfts: Vec<Nft> = Vec::new();
        for (_k, mut nft) in nfts_hash.into_iter(){
            let mut creators: Vec<String> = metadata_creators::table
                .select(metadata_creators::creator_address)
                .filter(metadata_creators::metadata_address.eq(&nft.address))
                .load(&conn)
                .unwrap();
            
                nft.creators.append(&mut creators);
                all_nfts.push(nft);
        }

        all_nfts
    }

    fn nft(&self, #[graphql(description = "Address of NFT")] address: String) -> Nft {
        let conn = self.db.get().unwrap();
        let rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(address))
            .limit(1)
            .load(&conn)
            .unwrap();

        // rows.pop().map(Into::into)
        Nft {
            address: rows[0].address.to_string(),
            name: rows[0].name.to_string(),
            symbol: rows[0].symbol.to_string(),
            uri: rows[0].uri.to_string(),
            seller_fee_basis_points: rows[0].seller_fee_basis_points,
            update_authority_address: "".to_owned(),
            mint_address: rows[0].mint_address.to_string(),
            primary_sale_happened: rows[0].primary_sale_happened,
            is_mutable: rows[0].is_mutable,
            creators: vec![],
        }
    }
}
pub struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    fn buyNft(_buy_nft: BuyNft) -> FieldResult<Nft> {
        Ok(Nft {
            address: "abc123".to_owned(),
            name: "foo".to_owned(),
            symbol: "BAR".to_owned(),
            uri: "https://ipfs.web/abc".to_owned(),
            seller_fee_basis_points: 1000,
            update_authority_address: "xyz123".to_owned(),
            mint_address: "efg890".to_owned(),
            primary_sale_happened: false,
            is_mutable: true,
            creators: vec![],
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create(db: Pool) -> Schema {
    Schema::new(QueryRoot { db }, MutationRoot {}, EmptySubscription::new())
}
