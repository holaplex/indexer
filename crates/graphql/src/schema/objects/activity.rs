use std::fmt;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLEnum)]
pub enum ActivityType {
    ListingCreated,
    OfferCreated,
    ListingCanceled,
    OfferCanceled,
    Purchase,
    Sales,
}

impl fmt::Display for ActivityType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
