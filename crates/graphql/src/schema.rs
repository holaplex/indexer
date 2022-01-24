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
}

impl<'a> From<models::Metadata<'a>> for Nft {
    fn from(
        models::Metadata {
            address,
            name,
            symbol,
            uri,
            seller_fee_basis_points,
            update_authority_address,
            mint_address,
            primary_sale_happened,
            is_mutable,
            edition_nonce: _,
        }: models::Metadata,
    ) -> Self {
        Self {
            address: address.into_owned(),
            name: name.into_owned(),
            uri: uri.into_owned(),
            symbol: symbol.into_owned(),
            seller_fee_basis_points,
            update_authority_address: update_authority_address.into_owned(),
            mint_address: mint_address.into_owned(),
            primary_sale_happened,
            is_mutable,
        }
    }
}

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

        // Create mutable vector for all rows returned
        let mut all_rows: Vec<String> = Vec::new();

        // Iterate across creators passed into function
        for creator in creators.into_iter().flatten() {
            // Database stuff
            let mut rows: Vec<String> = metadata_creators::table
                .select(metadata_creators::metadata_address)
                .filter(metadata_creators::creator_address.eq(creator))
                .load(&conn)
                .unwrap();

            // Append found rows to all rows vector
            all_rows.append(&mut rows);
        }

        for ua in update_authority.into_iter().flatten() {
            // Database stuff
            let mut rows: Vec<String> = metadatas::table
                .select(metadatas::address)
                .filter(metadatas::update_authority_address.eq(ua))
                .load(&conn)
                .unwrap();

            // Append found rows to all rows vector
            all_rows.append(&mut rows);
        }

        // now find all nfts
        let rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(any(all_rows)))
            .load(&conn)
            .unwrap();

        // Cast Models::Metadata to Nft and return
        rows.into_iter().map(Into::into).collect()
    }

    fn nft(&self, #[graphql(description = "Address of NFT")] address: String) -> Option<Nft> {
        let conn = self.db.get().unwrap();
        let mut rows: Vec<models::Metadata> = metadatas::table
            .select(metadatas::all_columns)
            .filter(metadatas::address.eq(address))
            .limit(1)
            .load(&conn)
            .unwrap();

        rows.pop().map(Into::into)
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
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create(db: Pool) -> Schema {
    Schema::new(QueryRoot { db }, MutationRoot {}, EmptySubscription::new())
}
