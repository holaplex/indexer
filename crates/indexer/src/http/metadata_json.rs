use std::fmt::{self, Debug, Display};

use cid::Cid;
use indexer_core::{
    db::{
        insert_into,
        models::{
            File as DbFile, MetadataAttributeWrite, MetadataCollection,
            MetadataJson as DbMetadataJson,
        },
        select,
        tables::{attributes, files, metadata_collections, metadata_jsons},
        Connection,
    },
    hash::HashMap,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use super::{client::ArTxid, Client};
use crate::prelude::*;

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

#[derive(Serialize, Deserialize, Debug)]
struct File {
    uri: Option<String>,
    #[serde(rename = "type")]
    ty: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Creator {
    address: String,
    share: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Property {
    files: Option<Vec<File>>,
    category: Option<String>,
    creators: Option<Vec<Creator>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum ValueDataType {
    Number(i64),
    String(String),
}

impl Display for ValueDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Number(n) => Display::fmt(n, f),
            Self::String(s) => Display::fmt(s, f),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    trait_type: Option<String>,
    value: Option<ValueDataType>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Collection {
    name: Option<String>,
    family: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetadataJson {
    name: String,
    symbol: Option<String>,
    description: Option<String>,
    seller_fee_basis_points: i64,
    image: Option<String>,
    animation_url: Option<String>,
    collection: Option<Collection>,
    attributes: Option<Vec<Attribute>>,
    external_url: Option<String>,
    properties: Option<Property>,
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
        indexer_core::util::duration_hhmmssfff(end_time - start_time)
    );

    Ok(json)
}

async fn try_locate_json(
    client: &Client,
    url: &Url,
    id: &AssetIdentifier,
    meta_key: Pubkey,
) -> Result<(MetadataJson, Vec<u8>)> {
    let http_client = reqwest::Client::new();
    let mut resp = None;

    for (url, fingerprint) in id
        .ipfs
        .map(|c| (client.ipfs_link(&c), c.to_bytes()))
        .into_iter()
        .chain(id.arweave.map(|t| (client.arweave_link(&t), t.0.to_vec())))
    {
        let url_str = url.as_ref().map_or("???", Url::as_str).to_owned();

        match fetch_json(http_client.clone(), meta_key, url).await {
            Ok(j) => {
                debug!("Using fetch from {:?} for metadata {}", url_str, meta_key);
                resp = Some((j, fingerprint));
            },
            Err(e) => warn!(
                "Metadata fetch {:?} for {} failed: {:?}",
                url_str, meta_key, e
            ),
        }
    }

    Ok(if let Some(r) = resp {
        r
    } else {
        (
            fetch_json(http_client, meta_key, Ok(url.clone()))
                .await
                .with_context(|| {
                    format!(
                        "Last-resort metadata fetch {:?} for {} failed",
                        url.as_str(),
                        meta_key,
                    )
                })?,
            vec![],
        )
    })
}

fn process_files(db: &Connection, addr: &str, files: Option<Vec<File>>) -> Result<()> {
    for File { uri, ty } in files.unwrap_or_else(Vec::new) {
        let (uri, ty) = if let Some(v) = uri.zip(ty) {
            v
        } else {
            debug!("Skipping malformed file in JSON");
            continue;
        };

        let row = DbFile {
            metadata_address: Borrowed(addr),
            uri: Owned(uri),
            file_type: Owned(ty),
        };

        insert_into(files::table)
            .values(&row)
            .on_conflict_do_nothing()
            .execute(db)
            .context("Failed to insert file!")?;
    }

    Ok(())
}

#[inline]
fn process_attributes(
    db: &Connection,
    addr: &str,
    attributes: Option<Vec<Attribute>>,
) -> Result<()> {
    for Attribute { trait_type, value } in attributes.unwrap_or_else(Vec::new) {
        let row = MetadataAttributeWrite {
            metadata_address: Borrowed(addr),
            trait_type: trait_type.map(Owned),
            value: value.as_ref().map(|v| Owned(v.to_string())),
        };

        insert_into(attributes::table)
            .values(&row)
            .on_conflict_do_nothing()
            .execute(db)
            .context("Failed to insert attribute!")?;
    }

    Ok(())
}

#[inline]
fn process_collection(db: &Connection, addr: &str, collection: Option<Collection>) -> Result<()> {
    if let Some(Collection { name, family }) = collection {
        let row = MetadataCollection {
            metadata_address: Borrowed(addr),
            name: name.map(Owned),
            family: family.map(Owned),
        };

        insert_into(metadata_collections::table)
            .values(&row)
            .on_conflict_do_nothing()
            .execute(db)
            .context("Failed to insert collection!")?;
    }

    Ok(())
}

pub async fn process<'a>(client: &Client, meta_key: Pubkey, uri_str: String) -> Result<()> {
    let url = Url::parse(&uri_str).context("Couldn't parse metadata JSON URL")?;
    let id = AssetIdentifier::new(&url);

    let possible_fingerprints: Vec<_> = id
        .ipfs
        .iter()
        .map(Cid::to_bytes)
        .chain(id.arweave.map(|a| a.0.to_vec()))
        .collect();
    let addr = bs58::encode(meta_key).into_string();

    let is_present = client
        .db()
        .run({
            let addr = addr.clone();
            move |db| {
                select(exists(
                    metadata_jsons::table.filter(
                        metadata_jsons::metadata_address
                            .eq(addr)
                            .and(metadata_jsons::fingerprint.eq(any(possible_fingerprints))),
                    ),
                ))
                .get_result(db)
            }
        })
        .await
        .context("Failed to check for already-indexed metadata JSON")?;

    if is_present {
        debug!("Skipping already-indexed metadata JSON for {}", meta_key);

        return Ok(());
    }

    debug!("{:?} -> {:?}", url.as_str(), id);

    let (json, fingerprint) = try_locate_json(client, &url, &id, meta_key).await?;

    let raw_content: serde_json::Value =
        serde_json::value::to_value(&json).context("Failed to upcast metadata JSON")?;

    let MetadataJson {
        description,
        image,
        animation_url,
        external_url,
        ..
    } = json;

    let (files, category, _creators) = json.properties.map_or(
        (None, None, None),
        |Property {
             files,
             category,
             creators,
         }| (files, category, creators),
    );

    let row = DbMetadataJson {
        metadata_address: Owned(addr.clone()),
        fingerprint: Owned(fingerprint),
        updated_at: Local::now().naive_utc(),
        description: description.map(Owned),
        image: image.map(Owned),
        animation_url: animation_url.map(Owned),
        external_url: external_url.map(Owned),
        category: category.map(Owned),
        raw_content: Owned(raw_content),
    };

    client
        .db()
        .run(move |db| {
            insert_into(metadata_jsons::table)
                .values(&row)
                .on_conflict(metadata_jsons::metadata_address)
                .do_update()
                .set(&row)
                .execute(db)
                .context("Failed to insert metadata")?;

            // TODO: if the row updates the following functions do not clear the
            //       previous rows from the old metadata JSON:

            process_files(db, &addr, files)?;
            process_attributes(db, &addr, json.attributes)?;
            process_collection(db, &addr, json.collection)
        })
        .await
}
