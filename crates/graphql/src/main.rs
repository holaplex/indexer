//! GraphQL server to read from `holaplex-indexer`

#![deny(
    clippy::disallowed_method,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{dev::ConnectionInfo, http, web, App, Error, HttpResponse, HttpServer};
use indexer_core::{
    assets::AssetProxyArgs,
    chrono::{Duration, Local},
    clap,
    clap::Parser,
    db,
    db::Pool,
    meilisearch,
    prelude::*,
    util::duration_hhmmssfff,
    ServerOpts,
};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
// TODO: use nonblocking once we upgrade past 1.9
use solana_client::rpc_client::RpcClient;

use crate::schema::{AppContext, Schema};

mod schema;

#[derive(Debug, Parser)]
struct Opts {
    #[clap(flatten)]
    server: ServerOpts,

    #[clap(flatten)]
    db: db::ConnectArgs,

    #[clap(long, env)]
    twitter_bearer_token: Option<String>,

    #[clap(flatten)]
    asset_proxy: AssetProxyArgs,

    #[clap(flatten)]
    search: meilisearch::Args,

    #[clap(long, env)]
    solana_endpoint: String,

    #[clap(long, env, use_value_delimiter(true))]
    follow_wallets_exclusions: Vec<String>,

    #[clap(long, env, use_value_delimiter(true))]
    featured_listings_auction_houses: Vec<String>,

    #[clap(long, env, use_value_delimiter(true))]
    featured_listings_seller_exclusions: Vec<String>,

    #[clap(long, env, use_value_delimiter(true))]
    marketplaces_store_address_exclusions: Vec<String>,

    #[clap(long, env)]
    pre_query_search_limit: usize,
}

struct GraphiqlData {
    uri: String,
}

struct RedirectData {
    route: &'static str,
    new_route: &'static str,
}

pub(crate) struct SharedData {
    schema: Schema,
    pub db: Arc<Pool>,
    pub asset_proxy: AssetProxyArgs,
    pub twitter_bearer_token: String,
    pub search: meilisearch::client::Client,
    pub rpc: RpcClient,
    pub follow_wallets_exclusions: Vec<String>,
    pub featured_listings_auction_houses: Vec<String>,
    pub featured_listings_seller_exclusions: Vec<String>,
    pub marketplaces_store_address_exclusions: Vec<String>,
    pub pre_query_search_limit: usize,
}

#[allow(clippy::unused_async)]
async fn graphiql(data: web::Data<GraphiqlData>) -> HttpResponse {
    let html = graphiql_source(&data.uri, None);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[allow(clippy::unused_async)]
async fn redirect_version(data: web::Data<RedirectData>) -> HttpResponse {
    HttpResponse::MovedPermanently()
        .insert_header(("Location", data.new_route))
        .body(format!(
            "API route {} deprecated, please use {}",
            data.route, data.new_route
        ))
}

async fn graphql(
    data: web::Data<SharedData>,
    req: web::Json<GraphQLRequest>,
    conn: ConnectionInfo,
) -> Result<HttpResponse, Error> {
    let ctx = AppContext::new(data.clone().into_inner());
    let start = Local::now();

    let resp = req.execute(&data.schema, &ctx).await;
    let end = Local::now();
    let duration = end - start;
    info!(
        "host={:?}, remote_addr={:?}, peer_addr={:?}",
        conn.host(),
        conn.realip_remote_addr().unwrap_or(&String::new()),
        conn.peer_addr().unwrap_or(&String::new())
    );
    if duration > Duration::milliseconds(5000) {
        #[derive(serde::Deserialize)]
        struct Data {
            #[serde(default)]
            query: serde_json::Value,
            #[serde(default)]
            operation_name: serde_json::Value,
            #[serde(default)]
            variables: serde_json::Value,
        }

        match serde_json::to_value(&req).and_then(serde_json::from_value) {
            Ok(Data {
                query,
                operation_name,
                variables,
            }) => warn!(
                "Long graphql request query={}, operation={:?}, variables={}, duration={}",
                query,
                operation_name,
                variables,
                duration_hhmmssfff(duration),
            ),
            Err(e) => error!("Failed to format long query for printing: {}", e),
        }
    }

    Ok(HttpResponse::Ok().json(&resp))
}

fn main() {
    indexer_core::run(|| {
        let opts = Opts::parse();
        debug!("{:#?}", opts);
        let Opts {
            server,
            db,
            twitter_bearer_token,
            asset_proxy,
            search,
            solana_endpoint,
            follow_wallets_exclusions,
            featured_listings_auction_houses,
            featured_listings_seller_exclusions,
            marketplaces_store_address_exclusions,
            pre_query_search_limit,
        } = opts;

        let (addr,) = server.into_parts();
        info!("Listening on {}", addr);

        let twitter_bearer_token = twitter_bearer_token.unwrap_or_else(String::new);

        // TODO: db_ty indicates if any actions that mutate the database can be run
        let db::ConnectResult {
            pool,
            ty: _,
            migrated: _,
        } = db::connect(db, db::ConnectMode::Read).context("Failed to connect to Postgres")?;
        let db = Arc::new(pool);
        let search = search.into_client();
        let rpc = RpcClient::new(solana_endpoint);

        let shared = web::Data::new(SharedData {
            schema: schema::create(),
            db,
            asset_proxy,
            twitter_bearer_token,
            search,
            rpc,
            follow_wallets_exclusions,
            featured_listings_auction_houses,
            featured_listings_seller_exclusions,
            marketplaces_store_address_exclusions,
            pre_query_search_limit,
        });

        let version_extension = "/v1";

        let redirect_data = web::Data::new(RedirectData {
            route: "/v0",
            new_route: "/v1",
        });

        // Should look something like "/..."
        let graphiql_data = web::Data::new(GraphiqlData {
            uri: version_extension.to_owned(),
        });
        assert!(graphiql_data.uri.starts_with('/'));

        actix_web::rt::System::new()
            .block_on(
                HttpServer::new(move || {
                    App::new()
                        .wrap(
                            Cors::default()
                                .allow_any_origin()
                                .allowed_methods(vec!["GET", "POST"])
                                .allowed_headers(vec![
                                    http::header::AUTHORIZATION,
                                    http::header::ACCEPT,
                                ])
                                .allowed_header(http::header::CONTENT_TYPE)
                                .max_age(3600),
                        )
                        .service(
                            web::resource(version_extension)
                                .app_data(shared.clone())
                                .route(web::post().to(graphql)),
                        )
                        .service(
                            web::resource(redirect_data.route)
                                .app_data(redirect_data.clone())
                                .to(redirect_version),
                        )
                        .service(
                            web::resource("/graphiql")
                                .app_data(graphiql_data.clone())
                                .route(web::get().to(graphiql)),
                        )
                })
                .bind(addr)?
                .run(),
            )
            .context("Actix server failed to run")
    });
}
