use scalars::Volume;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct MintStats {
    pub mint: String,
    pub floor: Volume,
    pub average: Volume,
    pub volume_24hr: Volume,
    pub count: Volume,
}
