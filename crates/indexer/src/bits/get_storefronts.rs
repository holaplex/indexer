use std::env;

use indexer_core::{
    db::{insert_into, models::Storefront, tables::storefronts, PooledConnection},
    hash::{HashMap, HashSet},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{prelude::*, Client, Job, ThreadPoolHandle};

#[derive(Serialize)]
struct Query {
    query: &'static str,
    variables: HashMap<&'static str, Value>,
}

#[repr(transparent)]
#[derive(Debug, Deserialize)]
struct QueryResponse {
    data: QueryData,
}

#[repr(transparent)]
#[derive(Debug, Deserialize)]
struct QueryData {
    transactions: QueryTransactions,
}

#[derive(Debug, Deserialize)]
struct QueryTransactions {
    edges: Vec<QueryEdge>,
    #[serde(rename = "pageInfo")]
    page_info: QueryPageInfo,
}

#[derive(Debug, Deserialize)]
struct QueryEdge {
    cursor: String,
    node: QueryNode,
}

#[derive(Debug, Deserialize)]
struct QueryNode {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    owner: QueryOwner,
    tags: Vec<QueryTag>,
    block: Option<QueryBlock>,
}

#[derive(Debug, Deserialize)]
struct QueryOwner {
    #[allow(dead_code)]
    address: String,
}

#[derive(Debug, Deserialize)]
struct QueryBlock {
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct QueryTag {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct QueryPageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
}

const QUERY: &str = r#"query GetStorefronts($after: String, $first: Int) {
    transactions(
        tags: [{ name: "Arweave-App", values: ["holaplex"] }],
        after: $after,
        sort: HEIGHT_DESC,
        first: $first,
    ) {
        pageInfo {
            hasNextPage
        }

        edges {
            cursor

            node {
                id
                owner {
                    address
                }
                tags {
                    name
                    value
                }
                block {
                    timestamp
                }
            }
        }
    }
}"#;

const BATCH: usize = 1000;

fn process_tags(
    mut tags: HashMap<String, String>,
    updated_at: Option<NaiveDateTime>,
    db: &PooledConnection,
    handle: ThreadPoolHandle<'_>,
    known_pubkeys: &mut HashSet<Pubkey>,
) -> Result<()> {
    let owner = Pubkey::try_from(
        tags.remove("solana:pubkey")
            .ok_or_else(|| anyhow!("Missing storefront owner key"))?
            .as_str(),
    )
    .context("Failed to parse owner pubkey")?;

    if known_pubkeys.insert(owner) {
        let subdomain = tags
            .remove("holaplex:metadata:subdomain")
            .ok_or_else(|| anyhow!("Missing storefront subdomain"))?;
        let title = tags
            .remove("holaplex:metadata:page:title")
            .unwrap_or_else(String::new);
        let description = tags
            .remove("holaplex:metadata:page:description")
            .unwrap_or_else(String::new);
        let favicon_url = tags
            .remove("holaplex:metadata:favicon:url")
            .unwrap_or_else(String::new);
        let logo_url = tags
            .remove("holaplex:theme:logo:url")
            .unwrap_or_else(String::new);
        let banner_url = tags
            .remove("holaplex:theme:banner:url")
            .unwrap_or_else(String::new);

        let row = Storefront {
            owner_address: Owned(bs58::encode(owner).into_string()),
            subdomain: Owned(subdomain),
            title: Owned(title),
            description: Owned(description),
            favicon_url: Owned(favicon_url),
            logo_url: Owned(logo_url),
            banner_url: Some(Owned(banner_url)),
            updated_at,
        };

        insert_into(storefronts::table)
            .values(&row)
            .on_conflict(storefronts::owner_address)
            .do_update()
            .set(&row)
            .execute(db)
            .context("Failed to insert storefront")?;

        handle.push(Job::StoreOwner(owner));
    } else {
        // This isn't terribly useful on its own as a trace log
        // trace!("Skipping duplicate owner {:?}", owner);
    }

    Ok(())
}

async fn get_storefronts_async(client: &Client, handle: ThreadPoolHandle<'_>) -> Result<()> {
    let db = client.db()?;

    let http_client = reqwest::Client::new();
    let url = env::var("ARWEAVE_URL")
        .context("Couldn't get Arweave URL")
        .and_then(|s| Url::parse(&s).context("Couldn't parse Arweave URL"))
        .map(|mut u| {
            u.set_path("/graphql");
            u
        })?;
    let mut after = String::new();
    let mut known_pubkeys = HashSet::default();

    loop {
        let QueryResponse {
            data:
                QueryData {
                    transactions:
                        QueryTransactions {
                            edges,
                            page_info: QueryPageInfo { has_next_page },
                        },
                },
        } = http_client
            .post(url.clone())
            .header("Content-Type", "application/json")
            .json(&Query {
                query: QUERY,
                variables: [
                    ("after", Value::String(after.clone())),
                    ("first", Value::Number(BATCH.into())),
                ]
                .into_iter()
                .collect(),
            })
            .send()
            .await
            .context("Arweave GraphQL request failed")?
            .json()
            .await
            .context("Couldn't parse Arweave GraphQL response")?;

        let mut next_after = None;

        for edge in edges {
            process_tags(
                edge.node
                    .tags
                    .into_iter()
                    .map(|QueryTag { name, value }| (name, value))
                    .collect(),
                edge.node
                    .block
                    .map(|b| NaiveDateTime::from_timestamp(b.timestamp, 0)),
                &db,
                handle,
                &mut known_pubkeys,
            )
            .map_err(|e| error!("{:?}", e))
            .ok();

            next_after = Some(edge.cursor);
        }

        if !has_next_page {
            break;
        }

        match next_after {
            Some(a) if a == after => return Err(anyhow!("Arweave fetch got stuck in a loop")),
            Some(a) => after = a,
            None => {
                warn!("Got zero edges in a request");
                break;
            },
        }
    }

    Ok(())
}

pub fn run(client: &Client, handle: ThreadPoolHandle) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async executor")?
        .block_on(get_storefronts_async(client, handle))
}
