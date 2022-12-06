//! Fast hash tables using `hashbrown` and the `ahash` hasher

pub extern crate dashmap;
pub extern crate hashbrown;

/// `hashbrown::HashMap` alias
#[allow(clippy::module_name_repetitions)]
pub type HashMap<K, V> = hashbrown::HashMap<K, V>;
/// `hashbrown::HashSet` alias
#[allow(clippy::module_name_repetitions)]
pub type HashSet<V> = hashbrown::HashSet<V>;
/// `dashmap::DashMap` alias that uses `ahash`
pub type DashMap<K, V> = dashmap::DashMap<K, V, ahash::RandomState>;
/// `dashmap::DashSet` alias that uses `ahash`
pub type DashSet<V> = dashmap::DashSet<V, ahash::RandomState>;
