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

#[derive(GraphQLInputObject)]
#[graphql(description = "Buy a NFT")]
struct BuyNft {
    transaction: String,
}

pub struct QueryRoot;

#[juniper::graphql_object]
impl QueryRoot {
    fn nfts(#[graphql(description = "Address of NFT")] address: Option<String>) -> Vec<Nft> {
        let mut x: Vec<Nft> = Vec::new();

        x.push(Nft {
            address: "abc123".to_owned(),
            name: "foo".to_owned(),
            symbol: "BAR".to_owned(),
            uri: "https://ipfs.web/abc".to_owned(),
            seller_fee_basis_points: 1000,
            update_authority_address: "xyz123".to_owned(),
            mint_address: "efg890".to_owned(),
            primary_sale_happened: false,
            is_mutable: true,
        });

        x.push(Nft {
            address: "xyz123".to_owned(),
            name: "foo".to_owned(),
            symbol: "BAR".to_owned(),
            uri: "https://ipfs.web/abc".to_owned(),
            seller_fee_basis_points: 1000,
            update_authority_address: "xyz123".to_owned(),
            mint_address: "efg890".to_owned(),
            primary_sale_happened: false,
            is_mutable: true,
        });

        x.push(Nft {
            address: "abc".to_owned(),
            name: "foo".to_owned(),
            symbol: "BAR".to_owned(),
            uri: "https://ipfs.web/abc".to_owned(),
            seller_fee_basis_points: 1000,
            update_authority_address: "xyz123".to_owned(),
            mint_address: "efg890".to_owned(),
            primary_sale_happened: false,
            is_mutable: true,
        });

        x.push(Nft {
            address: "123".to_owned(),
            name: "foo".to_owned(),
            symbol: "BAR".to_owned(),
            uri: "https://ipfs.web/abc".to_owned(),
            seller_fee_basis_points: 1000,
            update_authority_address: "xyz123".to_owned(),
            mint_address: "efg890".to_owned(),
            primary_sale_happened: false,
            is_mutable: true,
        });

        if let Some(address) = address {
            let y: Vec<Nft> = x.into_iter().filter(|xx| xx.address.eq(&address)).collect();
            return y;
        }
        return x;
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

pub fn create() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}
