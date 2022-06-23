use std::fmt::{self, Debug, Display};

use indexer_core::{
    assets::{proxy_url, proxy_url_hinted, AssetIdentifier},
    db::{
        insert_into,
        models::{
            File as DbFile, MetadataAttributeWrite, MetadataCollection,
            MetadataJson as DbMetadataJson,
        },
        tables::{
            attributes, files, metadata_collection_keys, metadata_collections, metadata_creators,
            metadata_jsons, metadatas, twitter_handle_name_services,
        },
        update, Connection,
    },
    hash::HashMap,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Client;
use crate::{
    prelude::*,
    search_dispatch::{CollectionDocument, MetadataDocument},
};

type SlotInfo = (i64, i64);

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
    Minimal {
        value: MetadataJsonMinimal,
        full_err: serde_json::Error,
    },
}

async fn fetch_json(
    client: &Client,
    meta_key: Pubkey,
    url: Result<Url>,
) -> Result<(Url, MetadataJsonResult)> {
    let start_time = Local::now();
    let url = url.context("Failed to create asset URL")?;

    let bytes = client
        .http()
        .run(|h| {
            let url = url.clone();
            async move { h.get(url).send().await?.bytes().await }
        })
        .await
        .context("Failed to download metadata JSON")?;

    let end_time = Local::now();

    trace!(
        "Metadata JSON URI {:?} for {} fetched in {}",
        url.as_str(),
        meta_key,
        indexer_core::util::duration_hhmmssfff(end_time - start_time)
    );

    let full_err;
    match serde_json::from_slice(&bytes) {
        Ok(f) => return Ok((url, MetadataJsonResult::Full(f))),
        Err(e) => {
            trace!(
                "Failed to parse full metadata JSON for {:?}: {:?}",
                url.as_str(),
                e
            );
            full_err = e;
        },
    };

    match serde_json::from_slice(&bytes) {
        Ok(value) => return Ok((url, MetadataJsonResult::Minimal { value, full_err })),
        Err(e) => {
            trace!(
                "Failed to parse minimal metadata JSON for {:?}: {:?}",
                url.as_str(),
                e
            );
        },
    };

    Err(anyhow!(
        "Failed to parse JSON response from {:?}",
        url.as_str()
    ))
}

async fn try_locate_json(
    client: &Client,
    id: &AssetIdentifier<'_>,
    meta_key: Pubkey,
) -> Result<Option<(MetadataJsonResult, Vec<u8>, Url)>> {
    // Set to true for fallback
    const TRY_LAST_RESORT: bool = false;
    // Set to true to fetch links with no fingerprint
    const FETCH_NON_PERMAWEB: bool = true;

    let mut resp = Ok(None);

    for (fingerprint, hint) in id.fingerprints_hinted() {
        let url = if let Some(hint) = hint {
            proxy_url_hinted(client.proxy_args(), id, hint, None)
                .map(|u| u.unwrap_or_else(|| unreachable!()))
        } else if FETCH_NON_PERMAWEB {
            Ok(id.url.clone())
        } else {
            continue;
        };
        let url_str = url.as_ref().map_or("???", Url::as_str).to_owned();

        match fetch_json(client, meta_key, url).await {
            Ok((url, json)) => {
                trace!("Using fetch from {:?} for metadata {}", url_str, meta_key);
                resp = Ok(Some((json, fingerprint, url)));
                break;
            },
            Err(e) => {
                warn!(
                    "Metadata fetch {:?} for {} failed: {:?}",
                    url_str, meta_key, e
                );

                resp = Err(());
            },
        }
    }

    Ok(match resp {
        Ok(Some((res, fingerprint, url))) => Some((res, fingerprint.into_owned(), url)),
        Ok(None) => {
            trace!(
                "Not fetching unparseable url {:?} for {}",
                id.url.as_str(),
                meta_key
            );

            None
        },
        Err(()) if TRY_LAST_RESORT => {
            let (url, json) = fetch_json(client, meta_key, Ok(id.url.clone()))
                .await
                .with_context(|| {
                    format!(
                        "Last-resort metadata fetch {:?} for {} failed",
                        id.url.as_str(),
                        meta_key,
                    )
                })?;

            Some((json, vec![], url))
        },
        Err(()) => {
            bail!(
                "Cached metadata fetch {:?} for {} failed (not trying last-resort)",
                id.url.as_str(),
                meta_key
            )
        },
    })
}

async fn process_full(
    client: &Client,
    addr: String,
    first_verified_creator: Option<String>,
    fetch_uri: String,
    json: MetadataJson,
    fingerprint: Vec<u8>,
    slot_info: SlotInfo,
) -> Result<()> {
    let raw_content: Value =
        serde_json::value::to_value(&json).context("Failed to upcast metadata JSON")?;

    dispatch_metadata_document(client, false, addr.clone(), raw_content.clone())
        .await
        .context("Failed to dispatch upsert metadata document job")?;

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

    let (slot, write_version) = slot_info;
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
        fetch_uri: Owned(fetch_uri),
        slot,
        write_version,
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

            process_files(db, &addr, files, slot_info)?;
            process_attributes(
                db,
                &addr,
                first_verified_creator.as_deref(),
                json.attributes,
                slot_info,
            )?;
            process_collection(db, &addr, json.collection, slot_info)
        })
        .await
}

async fn process_minimal(
    client: &Client,
    addr: String,
    fetch_uri: String,
    json: MetadataJsonMinimal,
    fingerprint: Vec<u8>,
    full_err: serde_json::Error,
    (slot, write_version): SlotInfo,
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

    dispatch_metadata_document(client, false, addr.clone(), raw_content.clone())
        .await
        .context("Failed to dispatch upsert metadata document job")?;

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
        model: Some(Owned(format!("minimal ({})", full_err))),
        fetch_uri: Owned(fetch_uri),
        slot,
        write_version,
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

fn process_files(
    db: &Connection,
    addr: &str,
    files: Option<Vec<File>>,
    slot_info: SlotInfo,
) -> Result<()> {
    for File { uri, ty } in files.unwrap_or_else(Vec::new) {
        let (uri, ty) = if let Some(v) = uri.zip(ty) {
            v
        } else {
            trace!("Skipping malformed file in JSON");
            continue;
        };

        let (slot, write_version) = slot_info;
        let row = DbFile {
            metadata_address: Borrowed(addr),
            uri: Owned(uri),
            file_type: Owned(ty),
            slot,
            write_version,
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
    slot_info: SlotInfo,
) -> Result<()> {
    for Attribute { trait_type, value } in attributes.unwrap_or_else(Vec::new) {
        let (slot, write_version) = slot_info;
        let row = MetadataAttributeWrite {
            metadata_address: Borrowed(addr),
            trait_type: trait_type.map(Owned),
            value: value.as_ref().map(|v| Owned(v.to_string())),
            first_verified_creator: first_verified_creator.map(Borrowed),
            slot,
            write_version,
        };

        insert_into(attributes::table)
            .values(&row)
            .on_conflict((
                attributes::metadata_address,
                attributes::value,
                attributes::trait_type,
            ))
            .do_update()
            .set(&row)
            .execute(db)
            .context("Failed to insert attribute!")?;
    }

    Ok(())
}

#[inline]
fn process_collection(
    db: &Connection,
    addr: &str,
    collection: Option<Collection>,
    slot_info: SlotInfo,
) -> Result<()> {
    if let Some(Collection { name, family }) = collection {
        let (slot, write_version) = slot_info;
        let row = MetadataCollection {
            metadata_address: Borrowed(addr),
            name: name.map(Owned),
            family: family.map(Owned),
            slot,
            write_version,
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
    (slot, write_version): SlotInfo,
) -> Result<()> {
    client
        .db()
        .run(move |db| {
            update(
                attributes::table
                    .filter(attributes::metadata_address.eq(addr))
                    .filter(
                        attributes::slot.lt(slot).or(attributes::slot
                            .eq(slot)
                            .and(attributes::write_version.lt(write_version))),
                    ),
            )
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
    (slot, write_version): (u64, u64),
) -> Result<()> {
    let slot_info = (
        i64::try_from(slot).context("Slot was too big to store")?,
        i64::try_from(write_version).context("Write version was too big to store")?,
    );

    let url = match Url::parse(&uri_str) {
        Ok(u) => u,
        Err(e) => {
            // Don't return an error because this happens A Lot.
            trace!("Couldn't parse metadata URL: {:?}", e);
            return Ok(());
        },
    };
    let id = AssetIdentifier::new(&url);

    let addr = bs58::encode(meta_key).into_string();
    let existing_row = client
        .db()
        .run({
            let addr = addr.clone();
            move |db| {
                metadata_jsons::table
                    .filter(metadata_jsons::metadata_address.eq(addr))
                    .select((
                        metadata_jsons::fingerprint,
                        metadata_jsons::raw_content,
                        (metadata_jsons::slot, metadata_jsons::write_version),
                    ))
                    .first::<(Cow<[u8]>, Value, SlotInfo)>(db)
                    .optional()
            }
        })
        .await
        .context("Failed to check for already-indexed metadata JSON")?;

    let first_verified_creator =
        first_verified_creator.map(|address| bs58::encode(address).into_string());

    if let Some((fingerprint, json, existing_slot_info)) = existing_row {
        if existing_slot_info > slot_info || id.fingerprints_hinted().any(|(f, _)| fingerprint == f)
        {
            trace!(
                "Skipping already-indexed metadata JSON for {} (seen at slot_info={:?})",
                meta_key,
                existing_slot_info
            );

            reprocess_attributes(client, addr.clone(), first_verified_creator, slot_info).await?;

            dispatch_metadata_document(client, true, addr, json).await?;

            return Ok(());
        }
    }

    trace!("{:?} -> {:?}", url.as_str(), id);

    if let Some((json, fingerprint, url)) = try_locate_json(client, &id, meta_key).await? {
        match json {
            MetadataJsonResult::Full(value) => {
                process_full(
                    client,
                    addr,
                    first_verified_creator,
                    url.to_string(),
                    value,
                    fingerprint,
                    slot_info,
                )
                .await?;
            },
            MetadataJsonResult::Minimal { value, full_err } => {
                process_minimal(
                    client,
                    addr,
                    url.to_string(),
                    value,
                    fingerprint,
                    full_err,
                    slot_info,
                )
                .await?;
            },
        }
    }

    Ok(())
}

async fn dispatch_metadata_document(
    client: &Client,
    is_for_backfill: bool,
    addr: String,
    raw: Value,
) -> Result<()> {
    let raw = if let Value::Object(m) = raw {
        m
    } else {
        bail!("Metadata JSON content was not an object");
    };

    let image = raw.get("image").and_then(Value::as_str);

    let image = image
        .and_then(|i| Url::parse(i).ok())
        .and_then(|u| {
            let id = AssetIdentifier::new(&u);

            proxy_url(client.proxy_args(), &id, Some(("width", "200")))
                .map(|o| o.map(|u| u.to_string()))
                .transpose()
        })
        .or_else(|| image.map(|s| Ok(s.into())))
        .transpose()?;

    if let Ok((name, mint_address, collection_address, creator_address, creator_twitter_handle)) =
        client
            .db()
            .run({
                let addr = addr.clone();
                move |db| {
                    let (name, mint_address, collection_address) =
                        metadatas::table
                            .left_join(metadata_collection_keys::table.on(
                                metadatas::address.eq(metadata_collection_keys::metadata_address),
                            ))
                            .filter(metadatas::address.eq(&addr))
                            .select((
                                metadatas::name,
                                metadatas::mint_address,
                                metadata_collection_keys::collection_address.nullable(),
                            ))
                            .first(db)
                            .context("failed to load mint and name for search doc")?;

                    let (creator_address, creator_twitter_handle) = metadata_creators::table
                        .left_join(
                            twitter_handle_name_services::table
                                .on(metadata_creators::creator_address
                                    .eq(twitter_handle_name_services::wallet_address)),
                        )
                        .filter(metadata_creators::metadata_address.eq(&addr))
                        .filter(metadata_creators::verified.eq(true))
                        .filter(metadata_creators::position.eq(0))
                        .select((
                            metadata_creators::creator_address,
                            twitter_handle_name_services::twitter_handle.nullable(),
                        ))
                        .first(db)
                        .context("failed to load creators for search document")?;

                    Result::<(String, String, Option<String>, String, Option<String>)>::Ok((
                        name,
                        mint_address,
                        collection_address,
                        creator_address,
                        creator_twitter_handle,
                    ))
                }
            })
            .await
            .map_err(|e| warn!("Failed to get search document data for metadata: {:?}", e))
    {
        let document = MetadataDocument {
            name,
            mint_address,
            image,
            creator_address,
            creator_twitter_handle,
            collection_address: collection_address.clone(),
        };

        client
            .search()
            .upsert_metadata(is_for_backfill, addr, document)
            .await
            .context("Failed to dispatch metadata JSON document job")?;

        if let Some(addr) = collection_address {
            upsert_collection_metadata(client, addr, is_for_backfill)
                .await
                .context("failed to index collection metadata")?;
        }
    }

    Ok(())
}

async fn upsert_collection_metadata(
    client: &Client,
    mint_address: String,
    is_for_backfill: bool,
) -> Result<()> {
    let (address, name, image) = client
        .db()
        .run({
            let mint_address = mint_address.clone();
            move |db| {
                let (address, name, image) = metadatas::table
                    .inner_join(
                        metadata_jsons::table
                            .on(metadatas::address.eq(metadata_jsons::metadata_address)),
                    )
                    .filter(metadatas::mint_address.eq(&mint_address))
                    .select((metadatas::address, metadatas::name, metadata_jsons::image))
                    .first(db)?;

                Result::<(String, String, Option<String>)>::Ok((address, name, image))
            }
        })
        .await
        .context("failed to fetch collection metadata")?;
    let document = client
        .search()
        .get_document("collections".to_string(), address.clone())
        .await;

    if document.is_ok() {
        return Ok(());
    }

    let image = image
        .clone()
        .and_then(|i| Url::parse(&i).ok())
        .and_then(|u| {
            let id = AssetIdentifier::new(&u);

            proxy_url(client.proxy_args(), &id, Some(("width", "200")))
                .map(|o| o.map(|u| u.to_string()))
                .transpose()
        })
        .or_else(|| image.map(Ok))
        .transpose()?;

    client
        .search()
        .upsert_collection(is_for_backfill, address, CollectionDocument {
            name,
            image,
            mint_address,
        })
        .await
        .context("Failed to dispatch collection document job")?;

    Ok(())
}
