use std::{collections::HashMap, hash::Hash, sync::Arc};

use super::prelude::*;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to the database")]
    ConnectionFailed,
    #[error("Failed to process a model: {0}")]
    ModelConvert(#[from] Arc<indexer_core::error::Error>),
}

impl From<indexer_core::error::Error> for Error {
    fn from(e: indexer_core::error::Error) -> Self {
        Self::ModelConvert(Arc::new(e))
    }
}

pub type BatchResult<T> = Result<T, Error>;
pub type BatchMap<K, V> = HashMap<K, BatchResult<V>>;
pub type TryBatchMap<K, V> = BatchResult<BatchMap<K, V>>;
pub type Loader<K, V> = dataloader::non_cached::Loader<K, BatchResult<V>, Batcher>;

/// Helper trait for wrapping a value in [`Result`] if necessary
pub trait ResultLike<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E, F: Into<E>> ResultLike<T, E> for Result<T, F> {
    fn into_result(self) -> Result<T, E> {
        self.map_err(Into::into)
    }
}

impl<T, E> ResultLike<T, E> for T
where
    std::convert::Infallible: Into<E>,
{
    fn into_result(self) -> Result<T, E> {
        Result::<T, std::convert::Infallible>::Ok(self).into_result()
    }
}

/// Helper trait for merging results into a [`BatchFn`] value
pub trait BatchExtend: Default {
    type Element;

    fn extend(&mut self, element: Self::Element);
}

impl<T> BatchExtend for Option<T> {
    type Element = T;

    fn extend(&mut self, element: T) {
        *self = Some(element);
    }
}

impl<T> BatchExtend for Vec<T> {
    type Element = T;

    fn extend(&mut self, element: T) {
        self.push(element);
    }
}

/// Helper trait for collecting an iterator of key-value pairs into a
/// [`HashMap`] respecting optional- or multiple-value configurations
pub trait BatchIter<K, V> {
    fn batch<B: BatchExtend<Element = V>>(self, keys: &[K]) -> BatchMap<K, B>;
}

impl<
    K: Clone + Eq + Hash,
    KR: Into<K>,
    V,
    R: ResultLike<V, indexer_core::error::Error>,
    I: Iterator<Item = (KR, R)>,
> BatchIter<K, V> for I
{
    fn batch<B: BatchExtend<Element = V>>(self, keys: &[K]) -> BatchMap<K, B> {
        self.fold(
            keys.iter()
                .cloned()
                .map(|k| (k, Ok(B::default())))
                .collect::<HashMap<_, _>>(),
            |mut h, (k, v)| {
                let val = h.entry(k.into()).or_insert_with(|| Ok(B::default()));

                if let Ok(inner) = val.as_mut() {
                    match v.into_result() {
                        Ok(m) => inner.extend(m),
                        Err(e) => *val = Err(e.into()),
                    }
                }

                h
            },
        )
    }
}

/// Helper trait for implementing [`BatchFn`] with a return value of [`Result`]
#[async_trait]
pub trait TryBatchFn<K, V> {
    async fn load(&mut self, keys: &[K]) -> TryBatchMap<K, V>;
}

#[derive(Clone)]
pub struct Batcher(Arc<Pool>);

impl Batcher {
    #[must_use]
    pub fn new(pool: Arc<Pool>) -> Self {
        Self(pool)
    }

    pub fn db(&self) -> Result<indexer_core::db::PooledConnection, Error> {
        self.0.get().map_err(|_| Error::ConnectionFailed)
    }
}

#[async_trait]
impl<K: Clone + Eq + Hash + Sync, V> BatchFn<K, BatchResult<V>> for Batcher
where
    Batcher: TryBatchFn<K, V>,
{
    async fn load(&mut self, keys: &[K]) -> BatchMap<K, V> {
        match TryBatchFn::load(self, keys).await {
            Ok(m) => m,
            Err(e) => keys.iter().cloned().map(|k| (k, Err(e.clone()))).collect(),
        }
    }
}
