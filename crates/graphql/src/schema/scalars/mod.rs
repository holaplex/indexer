mod bigint;
mod public_key;

pub(self) mod prelude {
    pub use juniper::{graphql_scalar, ParseScalarResult, ParseScalarValue, Value};

    pub(super) use super::super::prelude::*;
}

pub mod markers {
    pub struct StoreConfig;
    pub struct TokenMint;
    /// Used to denote a public key field to graphql while not
    /// indicating a relationship to any other specific object
    pub struct Unspecified;
}

pub use bigint::{I64, U64};
pub use public_key::PublicKey;
