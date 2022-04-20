use std::{sync::Arc, time::Duration};

use indexer_core::{assets::AssetProxyArgs, clap};
use tokio::sync::Mutex;

use crate::{db::Pool, prelude::*};

/// Common arguments for internal HTTP indexer usage
#[derive(Debug, clap::Parser)]
#[allow(missing_copy_implementations)]
pub struct Args {
    #[clap(flatten)]
    asset_proxy: AssetProxyArgs,

    /// HTTP request timeout, in seconds
    #[clap(long, env = "HTTP_INDEXER_TIMEOUT")]
    timeout: f64,
}

/// Wrapper for handling networking logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    http: Mutex<(u8, reqwest::Client)>,
    asset_proxy: AssetProxyArgs,
    timeout: Duration,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if an invalid URL is given for `ipfs_cdn` or
    /// `arweave_cdn`.
    pub fn new_rc(db: Pool, args: Args) -> Result<Arc<Self>> {
        let Args {
            asset_proxy,
            timeout,
        } = args;

        let timeout = Duration::from_secs_f64(timeout);

        Ok(Arc::new(Self {
            db,
            http: Mutex::new((0, Self::build_client(timeout)?)),
            asset_proxy,
            timeout,
        }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    fn build_client(timeout: Duration) -> Result<reqwest::Client> {
        reqwest::ClientBuilder::new()
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

    /// Acquire an HTTP client
    ///
    /// # Errors
    /// This function does not generate errors, it simply passes any errors
    /// raised by the given closure through to the function return.
    #[inline]
    pub async fn http<F: std::future::Future<Output = reqwest::Result<T>>, T>(
        &self,
        f: impl FnOnce(reqwest::Client) -> F,
    ) -> Result<T> {
        let (hint, http) = self.http.lock().await.clone();

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
                    let (ref mut hint2, ref mut http) = *self.http.lock().await;

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

    /// Get a reference to the asset proxy arguments, used by
    /// [`proxy_url`](indexer_core::assets::proxy_url)
    #[inline]
    pub fn proxy_args(&self) -> &AssetProxyArgs {
        &self.asset_proxy
    }
}
