use std::{
    fmt::{self, Debug},
    time::SystemTime,
};

use cid::Cid;
use indexer_core::{
    db::{
        insert_into,
        models::{
            Attribute as DbAttribute, File as DbFile, MetadataCollection,
            MetadataJson as DbMetadataJson,
        },
        tables::{attributes, files, metadata_collections, metadata_jsons},
    },
    hash::HashMap,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
struct File {
    uri: String,
    r#type: String,
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

impl fmt::Display for ValueDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    name: Option<String>,
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
    image: String,
    animation_url: Option<String>,
    collection: Option<Collection>,
    attributes: Option<Vec<Attribute>>,
    external_url: Option<String>,
    properties: Option<Property>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
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

    let db = client.db()?;

    let response = http_client
        .get(uri_str)
        .send()
        .await
        .context("Metadata JSON request failed")?;

    let json = &response
        .json::<MetadataJson>()
        .await
        .context("Failed to parse!")?;

    let raw_content: serde_json::Value = serde_json::value::to_value(json).unwrap();

    let addr = bs58::encode(meta_key).into_string();

    let fingerprint;
    if id.ipfs.is_some() {
        fingerprint = id.ipfs.unwrap().to_bytes();
    } else {
        fingerprint = id.arweave.unwrap().0.to_vec();
    }
    let properties = json.properties.as_ref();

    let row = DbMetadataJson {
        metadata_address: Borrowed(&addr),
        fingerprint: Some(Borrowed(&fingerprint)),
        updated_at: Some(NaiveDateTime::from_timestamp(
            (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64)
                .try_into()
                .unwrap(),
            0,
        )),
        description: json.description.as_ref().map(Into::into),
        image: Some(Borrowed(&json.image)),
        animation_url: json.animation_url.as_ref().map(Into::into),
        external_url: json.external_url.as_ref().map(Into::into),
        category: match properties {
            Some(a) => a.category.as_ref().map(Into::into),
            None => None,
        },
        raw_content: Some(Borrowed(&raw_content)),
    };

    insert_into(metadata_jsons::table)
        .values(&row)
        .on_conflict_do_nothing()
        .execute(&db)
        .context("Failed to insert metadata")?;

    if let Some(a) = properties {
        match &a.files {
            Some(files) => {
                for File { uri, r#type } in files {
                    let row = DbFile {
                        metadata_address: Borrowed(&addr),
                        uri: Some(uri.to_string()),
                        file_type: Some(r#type.to_string()),
                    };
                    insert_into(files::table)
                        .values(&row)
                        .on_conflict_do_nothing()
                        .execute(&db)
                        .context("Failed to insert file!")?;
                }
            },
            None => {},
        }
    }

    let attributes = json.attributes.as_ref();
    if let Some(attributes) = attributes {
        for Attribute {
            name,
            trait_type,
            value,
        } in attributes
        {
            let row = DbAttribute {
                metadata_address: Borrowed(&addr),
                name: name.as_ref().map(|a| a.to_string()),
                value: match value {
                    Some(a) => match &a {
                        ValueDataType::String(val) => Some(val.to_string()),
                        ValueDataType::Number(val) => Some(val.to_string()),
                    },
                    None => None,
                },
                trait_type: trait_type.as_ref().map(|a| a.to_string()),
            };
            insert_into(attributes::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(&db)
                .context("Failed to insert attribute!")?;
        }
    }

    match &json.collection {
        Some(collection) => {
            let row = MetadataCollection {
                metadata_address: Borrowed(&addr),
                name: collection.name.as_ref().map(Into::into),
                family: collection.family.as_ref().map(Into::into),
            };
            insert_into(metadata_collections::table)
                .values(&row)
                .on_conflict_do_nothing()
                .execute(&db)
                .context("Failed to insert collection!")?;
        },
        None => {},
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
