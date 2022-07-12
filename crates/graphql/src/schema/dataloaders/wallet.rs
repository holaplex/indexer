use futures_util::future::join_all;
use indexer_core::db::queries::{self};
use objects::profile::TwitterProfile;

use super::prelude::*;
use crate::schema::{objects::wallet::Wallet, scalars::PublicKey};

#[async_trait]
impl TryBatchFn<String, Option<TwitterProfile>> for TwitterBatcher {
    async fn load(
        &mut self,
        screen_names: &[String],
    ) -> TryBatchMap<String, Option<TwitterProfile>> {
        let http_client = reqwest::Client::new();

        let twitter_users = screen_names.iter().map(|screen_name| {
            let http_client = &http_client;
            let _ = self.bearer();
            let url = self.proxy_url(screen_name);

            async move {
                http_client
                    .get(url.map_err(Error::model_convert)?)
                    .header("Accept", "application/json")
                    .send()
                    .await
                    .map_err(Error::model_convert)?
                    .json::<TwitterProfile>()
                    .await
                    .map_err(Error::model_convert)
            }
        });

        let twitter_users: Vec<_> = join_all(twitter_users).await;

        Ok(twitter_users
            .into_iter()
            .map(|r| r.map_err(|e| error!("Failed to load Twitter profile: {:?}", e)))
            .filter_map(Result::ok)
            .map(|u| (u.handle.clone(), u))
            .batch(screen_names))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Wallet>, Option<Wallet>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Wallet>],
    ) -> TryBatchMap<PublicKey<Wallet>, Option<Wallet>> {
        let conn = self.db()?;

        let twitter_handles = queries::twitter_handle_name_service::get_multiple(
            &conn,
            addresses.iter().map(ToString::to_string).collect(),
        )?;

        let wallets = twitter_handles.into_iter().fold(
            addresses
                .into_iter()
                .map(|a| (a.clone(), None))
                .collect::<HashMap<_, _>>(),
            |mut h,
             models::TwitterHandle {
                 wallet_address,
                 twitter_handle,
                 ..
             }| {
                *h.entry(wallet_address.into_owned().into()).or_insert(None) =
                    Some(twitter_handle.into_owned());

                h
            },
        );

        Ok(wallets
            .into_iter()
            .map(|(k, v)| (k.clone(), Wallet::new(k, v)))
            .batch(addresses))
    }
}
