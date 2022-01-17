use cid::Cid;
use indexer_core::hash::HashMap;
use reqwest::Url;
use serde::Deserialize;

use crate::{client::ArTxid, prelude::*, util, Client, ThreadPoolHandle};

#[derive(Debug, Clone, Copy)]
struct AssetIdentifier {
    ipfs: Option<Cid>,
    arweave: Option<ArTxid>,
}

impl AssetIdentifier {
    fn visit_url(url: &Url, mut f: impl FnMut(&str)) {
        Some(url.scheme())
            .into_iter()
            .chain(url.domain().into_iter().flat_map(|s| s.split('.')))
            .chain(Some(url.username()))
            .chain(url.password())
            .chain(Some(url.path()))
            .chain(url.path_segments().into_iter().flatten())
            .chain(url.query())
            .chain(url.fragment().into_iter().flat_map(|s| s.split('/')))
            .for_each(&mut f);

        url.query_pairs().for_each(|(k, v)| {
            f(k.as_ref());
            f(v.as_ref());
        });
    }

    fn try_ipfs(s: &str) -> Option<Cid> {
        s.try_into().ok()
    }

    fn try_arweave(s: &str) -> Option<ArTxid> {
        [
            base64::URL_SAFE,
            base64::URL_SAFE_NO_PAD,
            base64::STANDARD,
            base64::STANDARD_NO_PAD,
        ]
        .into_iter()
        .find_map(|c| {
            base64::decode_config(s.as_bytes(), c)
                .ok()
                .and_then(|v| v.try_into().ok())
                .map(ArTxid)
        })
    }

    fn advance_heuristic<T>(state: &mut Result<Option<T>, ()>, value: T) {
        match state {
            Ok(None) => *state = Ok(Some(value)),
            Ok(Some(_)) => *state = Err(()),
            Err(()) => (),
        }
    }

    fn new(url: &Url) -> Self {
        let mut ipfs = Ok(None);
        let mut arweave = Ok(None);

        Self::visit_url(url, |s| {
            if let Some(c) = Self::try_ipfs(s) {
                Self::advance_heuristic(&mut ipfs, c);
            }

            if let Some(t) = Self::try_arweave(s) {
                Self::advance_heuristic(&mut arweave, t);
            }
        });

        Self {
            ipfs: ipfs.ok().flatten(),
            arweave: arweave.ok().flatten(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MetadataJson {
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

async fn fetch_json(
    http_client: reqwest::Client,
    meta_key: Pubkey,
    url: Result<Url>,
) -> Result<MetadataJson> {
    let start_time = Local::now();
    let url = url.context("Failed to create asset URL")?;

    let json = http_client
        .get(url.clone())
        .send()
        .await
        .context("Metadata JSON request failed")?
        .json()
        .await
        .context("Failed to parse metadata JSON")?;

    let end_time = Local::now();

    debug!(
        "Metadata JSON URI {:?} for {} fetched in {}",
        url,
        meta_key,
        util::duration_hhmmssfff(end_time - start_time)
    );

    Ok(json)
}

async fn process_async<'a>(
    client: &Client,
    meta_key: Pubkey,
    uri_str: String,
    _handle: ThreadPoolHandle<'a>,
) -> Result<()> {
    let http_client = reqwest::Client::new();
    let url = Url::parse(&uri_str).context("Couldn't parse metadata JSON URL")?;

    let id = AssetIdentifier::new(&url);

    debug!("{:?} -> {:?}", url.as_str(), id);

    let mut json = None;

    for (url, handle) in id
        .ipfs
        .map(|c| client.ipfs_link(&c))
        .into_iter()
        .chain(id.arweave.map(|t| client.arweave_link(&t)))
        .map(|url| {
            (
                url.as_ref().ok().cloned(),
                tokio::spawn(fetch_json(http_client.clone(), meta_key, url)),
            )
        })
    {
        if json.is_some() {
            handle.abort();
            continue;
        }

        match handle.await? {
            Ok(j) => {
                debug!(
                    "Using fetch from {:?} for metadata {}",
                    url.as_ref().map_or("???", Url::as_str),
                    meta_key
                );
                json = Some(j);
            },
            Err(e) => warn!(
                "Metadata fetch {:?} for {} failed: {:?}",
                url.as_ref().map_or("???", Url::as_str),
                meta_key,
                e
            ),
        }
    }

    let json = if let Some(j) = json {
        j
    } else {
        fetch_json(http_client, meta_key, Ok(url.clone()))
            .await
            .with_context(|| {
                format!(
                    "Last-resort metadata fetch {:?} for {} failed",
                    url.as_str(),
                    meta_key,
                )
            })?
    };

    trace!("{}: {:#?}", meta_key, json);

    Ok(())
}

pub fn process(
    client: &Client,
    meta_key: Pubkey,
    uri: String,
    handle: ThreadPoolHandle,
) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async executor")?
        .block_on(process_async(client, meta_key, uri, handle))
}
