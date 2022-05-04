//! Support module for managing a reqwest HTTP client.

use std::time::Duration;

pub use ::reqwest::*;
use indexer_core::{
    error::Result as IResult,
    prelude::{anyhow, error, warn, Context},
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Client {
    inner: Mutex<(u8, reqwest::Client)>,
    timeout: Duration,
}

impl Client {
    pub fn new(timeout: Duration) -> IResult<Self> {
        Ok(Self {
            inner: Mutex::new((0, Self::build_client(timeout)?)),
            timeout,
        })
    }

    fn build_client(timeout: Duration) -> IResult<reqwest::Client> {
        ClientBuilder::new()
            .timeout(timeout)
            .pool_idle_timeout(
                timeout
                    .checked_mul(2)
                    .ok_or_else(|| anyhow!("Arithmetic error setting pool idle timeout"))?,
            )
            .pool_max_idle_per_host(4)
            .build()
            .context("Failed to build HTTP client")
    }

    /// Acquire an HTTP client and execute a request with i
    ///
    /// # Errors
    /// This function does not generate errors, it simply passes any errors
    /// raised by the given closure through to the function return.
    #[inline]
    pub async fn run<F: std::future::Future<Output = Result<T>>, T>(
        &self,
        f: impl FnOnce(reqwest::Client) -> F,
    ) -> IResult<T> {
        let (hint, http) = self.inner.lock().await.clone();

        match f(http).await {
            Ok(v) => Ok(v),
            Err(e) => {
                if e.is_connect()
                    || !(e.is_redirect()
                        || e.is_status()
                        || e.is_timeout()
                        || e.is_body()
                        || e.is_decode())
                {
                    // Something may have happened, close the connection pool
                    let (ref mut hint2, ref mut http) = *self.inner.lock().await;

                    if *hint2 == hint {
                        warn!("Connection error detected, rotating HTTP client");

                        match Self::build_client(self.timeout) {
                            Ok(client) => {
                                *hint2 = hint2.wrapping_add(1);
                                *http = client;
                            },
                            Err(e) => error!("Failed to rotate HTTP client: {:?}", e),
                        }
                    }
                }

                Err(e).context("HTTP request failed")
            },
        }
    }
}
