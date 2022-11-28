use std::fmt;

use reqwest::Url;
use serde_json::Value;

use super::prelude::*;

#[derive(Debug, serde::Deserialize)]
pub struct CurrencyInfo {
    current_price: f64,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct SolanaNetwork;

#[graphql_object(Context = AppContext)]
impl SolanaNetwork {
    async fn tps(&self, ctx: &AppContext) -> FieldResult<i32> {
        let shared = ctx.shared.clone();

        tokio::task::spawn_blocking(move || {
            let samples = shared
                .rpc
                .get_recent_performance_samples(Some(1))
                .context("RPC call for recent performance samples failed")?;

            let sample = samples
                .first()
                .context("failed to get recent performance sample")?;

            let tps: i32 =
                (sample.num_transactions / u64::from(sample.sample_period_secs)).try_into()?;
            Ok(tps)
        })
        .await
        .expect("Blocking task panicked")
    }

    pub async fn price(&self, ctx: &AppContext, currency: Currency) -> FieldResult<f64> {
        let http = &ctx.shared.http;
        let endpoint = &ctx.shared.coingecko_endpoint;
        let url: Url =
            format!("{endpoint}/coins/markets?vs_currency={currency}&ids=solana").parse()?;

        let res = http
            .get(url)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json::<Vec<CurrencyInfo>>()
            .await?;

        Ok(res
            .first()
            .context("failed to get solana price")?
            .current_price)
    }
}

#[derive(Debug, Clone, GraphQLEnum)]
pub enum Currency {
    Btc,
    Eth,
    Ltc,
    Bch,
    Bnb,
    Eos,
    Xrp,
    Xlm,
    Link,
    Dot,
    Yfi,
    Usd,
    Aed,
    Ars,
    Aud,
    Bdt,
    Bhd,
    Bmd,
    Brl,
    Cad,
    Chf,
    Clp,
    Cny,
    Czk,
    Dkk,
    Eur,
    Gbp,
    Hkd,
    Huf,
    Idr,
    Ils,
    Inr,
    Jpy,
    Krw,
    Kwd,
    Lkr,
    Mmk,
    Mxn,
    Myr,
    Ngn,
    Nok,
    Nzd,
    Php,
    Pkr,
    Pln,
    Rub,
    Sar,
    Sek,
    Sgd,
    Thb,
    Try,
    Twd,
    Uah,
    Vef,
    Vnd,
    Zar,
    Xdr,
    Xag,
    Xau,
    Bits,
    Sats,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
