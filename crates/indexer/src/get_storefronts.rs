use std::{collections::HashMap, env};

use indexer_core::hash::HashSet;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{prelude::*, Job, ThreadPoolHandle};

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
    id: String,
    owner: QueryOwner,
    tags: Vec<QueryTag>,
}

#[derive(Debug, Deserialize)]
struct QueryOwner {
    address: String,
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
            }
        }
    }
}"#;

const BATCH: usize = 1000;

async fn get_storefronts_async(handle: ThreadPoolHandle<'_>) -> Result<()> {
    let client = reqwest::Client::new();
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
        } = client
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
            match edge
                .node
                .tags
                .iter()
                .find(|t| t.name == "solana:pubkey")
                .ok_or_else(|| {
                    anyhow!("Missing storefront owner key for Arweave storefront record")
                })
                .and_then(|o| {
                    Pubkey::try_from(o.value.as_str()).context("Failed to parse owner pubkey")
                }) {
                Ok(k) if known_pubkeys.insert(k) => handle.push(Job::StoreOwner(k)),
                Ok(k) => trace!("Skipping duplicate owner {:?}", k),
                Err(e) => error!("Failed to get store owner: {:?}", e),
            }

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

pub fn run(handle: ThreadPoolHandle) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async executor")?
        .block_on(get_storefronts_async(handle))
}
