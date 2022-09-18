//! Reusable query operations for common or complicated queries.

pub mod activities;
pub mod bonding_changes;
pub mod charts;
pub mod collections;
pub mod featured_listings;
pub mod feed_event;
pub mod genopets;
pub mod graph_connection;
pub mod listing_denylist;
pub mod metadata_edition;
pub mod metadatas;
pub mod nft_count;
pub mod spl_governance;
pub mod stats;
pub mod store_denylist;
pub mod twitter_handle_name_service;
pub mod reward_centers;
pub mod wallet;

pub use util::*;

mod util {
    use std::ops::Bound;

    use diesel::{
        expression::AsExpression,
        helper_types::{Gt, GtEq, Lt, LtEq},
        query_dsl::methods::FilterDsl,
        sql_types::SingleValue,
        Expression, ExpressionMethods,
    };

    /// Perform a range query on an expression
    pub fn handle_range<
        Q: FilterDsl<Gt<E, T>, Output = Q>
            + FilterDsl<GtEq<E, T>, Output = Q>
            + FilterDsl<Lt<E, T>, Output = Q>
            + FilterDsl<LtEq<E, T>, Output = Q>,
        E: Expression + Copy,
        T: AsExpression<E::SqlType>,
    >(
        mut query: Q,
        expr: E,
        (min, max): (Bound<T>, Bound<T>),
    ) -> Q
    where
        E::SqlType: SingleValue,
    {
        match min {
            Bound::Unbounded => (),
            Bound::Excluded(min) => query = query.filter(expr.gt(min)),
            Bound::Included(min) => query = query.filter(expr.ge(min)),
        }

        match max {
            Bound::Unbounded => (),
            Bound::Excluded(max) => query = query.filter(expr.lt(max)),
            Bound::Included(max) => query = query.filter(expr.le(max)),
        }

        query
    }
}
