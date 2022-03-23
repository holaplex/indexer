use std::{
    hash::{BuildHasher, Hash, Hasher},
    mem::ManuallyDrop,
    sync::Arc,
};

use crossbeam::queue::ArrayQueue;
use indexer_core::{ahash::RandomState, prelude::*};
use reqwest::{Client, Url};

const MAX_WIDTH: usize = 32;

#[derive(Debug)]
pub struct HttpClientCache {
    hasher_builder: RandomState,
    clients: [Arc<ArrayQueue<Client>>; MAX_WIDTH],
}

impl HttpClientCache {
    pub fn new(depth: usize) -> Self {
        Self {
            hasher_builder: RandomState::new(),
            clients: std::iter::repeat_with(|| Arc::new(ArrayQueue::new(depth)))
                .take(MAX_WIDTH)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn acquire(&self, url: &Url) -> Result<CachedClient> {
        let mut hasher = self.hasher_builder.build_hasher();
        url.host_str()
            .ok_or_else(|| anyhow!("Missing host identifier for cache"))?
            .hash(&mut hasher);
        let hash = hasher.finish();

        let idx: usize = hash
            .rem_euclid(
                self.clients
                    .len()
                    .try_into()
                    .unwrap_or_else(|_| unreachable!()),
            )
            .try_into()
            .unwrap_or_else(|_| unreachable!());
        // Safety: because of rem_euclid idx < self.clients.len()
        let q = unsafe { self.clients.get_unchecked(idx) };

        trace!("Checking out client for {:?} at slot {}", url, idx);

        let client = q.pop().unwrap_or_else(|| {
            trace!("No clients to check out in slot {}", idx);

            Client::new()
        });

        Ok(CachedClient {
            cache: q.clone(),
            client: ManuallyDrop::new(client),
        })
    }
}

#[derive(Debug)]
pub struct CachedClient {
    cache: Arc<ArrayQueue<Client>>,
    client: ManuallyDrop<Client>,
}

impl Drop for CachedClient {
    fn drop(&mut self) {
        match self
            .cache
            .push(unsafe { ManuallyDrop::take(&mut self.client) })
        {
            Ok(()) => (), // client was successfully moved out, do NOT drop
            Err(c) => {
                trace!("Queue depth exceeded, not checking client back in");

                // IMPORTANT: c is a copy of the original client, we want to
                //            only drop the original
                std::mem::forget(c);
                unsafe {
                    ManuallyDrop::drop(&mut self.client);
                }
            },
        }
    }
}

impl AsRef<Client> for CachedClient {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}

impl std::borrow::Borrow<Client> for CachedClient {
    fn borrow(&self) -> &Client {
        &self.client
    }
}
impl std::ops::Deref for CachedClient {
    type Target = Client;

    fn deref(&self) -> &Client {
        &self.client
    }
}
