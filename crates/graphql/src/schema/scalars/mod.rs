mod bigint;
mod public_key;

pub(self) mod prelude {
    pub use juniper::{graphql_scalar, ParseScalarResult, ParseScalarValue, Value};

    pub(super) use super::super::prelude::*;
}

pub mod markers {
    pub struct StoreConfig;
    pub struct TokenMint;
    pub struct GovernedAccount;
    pub struct CommunityMint;
    pub struct GoverningTokenOwner;
    pub struct CouncilMint;
    pub struct GovernanceDelegate;
}

pub use bigint::{I64, U64};
pub use public_key::PublicKey;
