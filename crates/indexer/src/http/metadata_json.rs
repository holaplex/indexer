use std::{
    fmt::{self, Debug, Display},
    time::Duration,
};

use cid::Cid;
use indexer_core::{
    assets::AssetIdentifier,
    db::{
        insert_into,
        models::{
            File as DbFile, MetadataAttributeWrite, MetadataCollection,
            MetadataJson as DbMetadataJson,
        },
        select,
        tables::{attributes, files, metadata_collections, metadata_jsons},
        update, Connection,
    },
    hash::HashMap,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Client;
use crate::prelude::*;

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
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetadataJsonMinimal {
    #[serde(default)]
    name: Value,
    #[serde(default)]
    description: Value,
    #[serde(default)]
    image: Value,
    #[serde(default)]
    animation_url: Value,
    #[serde(default)]
    external_url: Value,
    #[serde(default)]
    category: Value,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

enum MetadataJsonResult {
    Full(MetadataJson),
    Minimal(MetadataJsonMinimal),
}

async fn fetch_json(
    http_client: reqwest::Client,
    meta_key: Pubkey,
    url: Result<Url>,
) -> Result<MetadataJsonResult> {
    let start_time = Local::now();
    let url = url.context("Failed to create asset URL")?;

    // TODO: what's a good timeout?
    let timeout = Duration::from_secs(10);

    let bytes = http_client
        .get(url.clone())
        .timeout(timeout)
        .send()
        .await
        .context("Metadata JSON request failed")?
        .bytes()
        .await
        .context("Failed to download metadata JSON")?;

    let end_time = Local::now();

    debug!(
        "Metadata JSON URI {:?} for {} fetched in {}",
        url.as_str(),
        meta_key,
        indexer_core::util::duration_hhmmssfff(end_time - start_time)
    );

    if let Ok(full) = serde_json::from_slice(&bytes).map_err(|e| {
        debug!(
            "Failed to parse full metadata JSON for {:?}: {:?}",
            url.as_str(),
            e
        );
    }) {
        Ok(MetadataJsonResult::Full(full))
    } else if let Ok(min) = serde_json::from_slice(&bytes).map_err(|e| {
        debug!(
            "Failed to parse minimal metadata JSON for {:?}: {:?}",
            url.as_str(),
            e
        );
    }) {
        Ok(MetadataJsonResult::Minimal(min))
    } else {
        Err(anyhow!(
            "Failed to parse JSON response from {:?}",
            url.as_str()
        ))
    }
}

async fn try_locate_json(
    client: &Client,
    url: &Url,
    id: &AssetIdentifier,
    meta_key: Pubkey,
) -> Result<(MetadataJsonResult, Vec<u8>)> {
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
        // Set to true for fallback
        const TRY_LAST_RESORT: bool = false;

        if TRY_LAST_RESORT {
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
        } else {
            bail!(
                "Cached metadata fetch {:?} for {} failed (not tryiing last-resort)",
                url.as_str(),
                meta_key
            )
        }
    })
}

async fn process_full(
    client: &Client,
    addr: String,
    first_verified_creator: Option<String>,
    json: MetadataJson,
    fingerprint: Vec<u8>,
) -> Result<()> {
    let raw_content: Value =
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
        model: Some(Borrowed("full")),
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
            process_attributes(
                db,
                &addr,
                first_verified_creator.as_deref(),
                json.attributes,
            )?;
            process_collection(db, &addr, json.collection)
        })
        .await
}

async fn process_minimal(
    client: &Client,
    addr: String,
    json: MetadataJsonMinimal,
    fingerprint: Vec<u8>,
) -> Result<()> {
    fn to_opt_string(v: &Value) -> Option<Cow<'static, str>> {
        v.as_str().map(|s| Owned(s.to_owned())).or_else(|| {
            if v.is_null() {
                None
            } else {
                Some(Owned(v.to_string()))
            }
        })
    }

    let raw_content: Value =
        serde_json::value::to_value(&json).context("Failed to upcast minimal metadata JSON")?;

    let MetadataJsonMinimal {
        name: _,
        description,
        image,
        animation_url,
        external_url,
        category,
        extra: _,
    } = json;

    let row = DbMetadataJson {
        metadata_address: Owned(addr.clone()),
        fingerprint: Owned(fingerprint),
        updated_at: Local::now().naive_utc(),
        description: to_opt_string(&description),
        image: to_opt_string(&image),
        animation_url: to_opt_string(&animation_url),
        external_url: to_opt_string(&external_url),
        category: to_opt_string(&category),
        raw_content: Owned(raw_content),
        model: Some(Borrowed("minimal")),
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
        })
        .await
        .context("Failed to insert minimal metadata")?;

    Ok(())
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
    first_verified_creator: Option<&str>,
    attributes: Option<Vec<Attribute>>,
) -> Result<()> {
    for Attribute { trait_type, value } in attributes.unwrap_or_else(Vec::new) {
        let row = MetadataAttributeWrite {
            metadata_address: Borrowed(addr),
            trait_type: trait_type.map(Owned),
            value: value.as_ref().map(|v| Owned(v.to_string())),
            first_verified_creator: first_verified_creator.map(Borrowed),
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

async fn reprocess_attributes(
    client: &Client,
    addr: String,
    first_verified_creator: Option<String>,
) -> Result<()> {
    client
        .db()
        .run(move |db| {
            update(attributes::table.filter(attributes::metadata_address.eq(addr)))
                .set(attributes::first_verified_creator.eq(first_verified_creator))
                .execute(db)
        })
        .await
        .context("Failed to update attributes")?;

    Ok(())
}

pub async fn process<'a>(
    client: &Client,
    meta_key: Pubkey,
    first_verified_creator: Option<Pubkey>,
    uri_str: String,
) -> Result<()> {
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

    let first_verified_creator =
        first_verified_creator.map(|address| bs58::encode(address).into_string());

    if is_present {
        debug!("Skipping already-indexed metadata JSON for {}", meta_key);

        // NOTE: For future reference, this introduces a situation with non-
        //       idempotent updates.  It is possible that with job retries, a
        //       sequence of two metadata jobs with differing values for
        //       first_verified_creator can write the wrong value.  If the first
        //       job fails, it will be eventually requeued after the second job.
        //       If the second job subsequently succeeds, then this reprocess
        //       function will be called by the first job and the first
        //       verified creator will be updated to an out-of-date value.
        reprocess_attributes(client, addr, first_verified_creator).await?;

        return Ok(());
    }

    debug!("{:?} -> {:?}", url.as_str(), id);

    let (json, fingerprint) = try_locate_json(client, &url, &id, meta_key).await?;

    match json {
        MetadataJsonResult::Full(f) => {
            process_full(client, addr, first_verified_creator, f, fingerprint).await
        },
        MetadataJsonResult::Minimal(m) => process_minimal(client, addr, m, fingerprint).await,
    }
}
