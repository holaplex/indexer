//! Fast hash tables using the `ahash` hasher

use ahash::RandomState;

/// `std::collections::HashMap` alias that uses `ahash`
#[allow(clippy::module_name_repetitions)]
pub type HashMap<K, V> = std::collections::HashMap<K, V, RandomState>;
/// `std::collections::HashSet` alias that uses `ahash`
#[allow(clippy::module_name_repetitions)]
pub type HashSet<V> = std::collections::HashSet<V, RandomState>;
/// `dashmap::DashMap` alias that uses `ahash`
pub type DashMap<K, V> = dashmap::DashMap<K, V, RandomState>;
/// `dashmap::DashSet` alias that uses `ahash`
pub type DashSet<V> = dashmap::DashSet<V, RandomState>;
