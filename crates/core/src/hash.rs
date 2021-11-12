use std::hash::BuildHasher;

use rand::{prelude::*, rngs::StdRng};
use seahash::SeaHasher;

/// # NOTE
/// This hasher is susceptible to hash-DoS attacks.  Use with care if ingesting
/// user data into a map with this hasher.
#[derive(Debug, Clone, Copy)]
pub struct SeaHasherBuilder([u64; 4]);

impl Default for SeaHasherBuilder {
    fn default() -> Self {
        let mut rng = StdRng::from_entropy();
        let mut seed = [0_u64; 4];
        rng.fill(&mut seed);

        Self(seed)
    }
}

impl BuildHasher for SeaHasherBuilder {
    type Hasher = SeaHasher;

    fn build_hasher(&self) -> SeaHasher {
        let [k1, k2, k3, k4] = self.0;
        SeaHasher::with_seeds(k1, k2, k3, k4)
    }
}

pub type HashMap<K, V> = std::collections::HashMap<K, V, SeaHasherBuilder>;
pub type HashSet<V> = std::collections::HashSet<V, SeaHasherBuilder>;
pub type DashMap<K, V> = dashmap::DashMap<K, V, SeaHasherBuilder>;
pub type DashSet<V> = dashmap::DashSet<V, SeaHasherBuilder>;
