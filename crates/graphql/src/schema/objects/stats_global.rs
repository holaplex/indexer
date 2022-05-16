use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct GlobalStats {
    #[graphql(description = "Approximate number of NFTs being minted per second.")]
    pub mint_rate: Option<f64>,

    #[graphql(description = "Approximate number of NFTs being bought from listings per second.")]
    pub purchase_rate: Option<f64>,
}
