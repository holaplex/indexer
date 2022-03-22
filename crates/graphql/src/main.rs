//! GraphQL server to read from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{future::Future, pin::Pin, sync::Arc};

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer};
use indexer_core::{clap, clap::Parser, db, db::Pool, prelude::*, ServerOpts};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

use crate::schema::{AppContext, Schema};

mod schema;

#[derive(Parser)]
struct Opts {
    #[clap(flatten)]
    server: ServerOpts,

    #[clap(long, env)]
    twitter_bearer_token: Option<String>,

    #[clap(long, env)]
    asset_proxy_endpoint: String,

    #[clap(long, env)]
    asset_proxy_count: u8,
}

fn graphiql(uri: String) -> impl Fn() -> HttpResponse + Clone {
    move || {
        let html = graphiql_source(&uri, None);

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    }
}

fn graphql(
    db_pool: Arc<Pool>,
    shared: Arc<SharedData>,
) -> impl Fn(
    web::Data<Arc<Schema>>,
    web::Json<GraphQLRequest>,
) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
+ Clone {
    move |st: web::Data<Arc<Schema>>, data: web::Json<GraphQLRequest>| {
        let pool = Arc::clone(&db_pool);
        let shared = Arc::clone(&shared);

        Box::pin(async move {
            let ctx = AppContext::new(pool, shared);
            let res = data.execute(&st, &ctx).await;

            let json = serde_json::to_string(&res)?;

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(json))
        })
    }
}

pub(crate) struct SharedData {
    pub asset_proxy_endpoint: String,
    pub asset_proxy_count: u8,
    pub twitter_bearer_token: String,
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            server,
            twitter_bearer_token,
            asset_proxy_endpoint,
            asset_proxy_count,
        } = Opts::parse();

        let (addr,) = server.into_parts();
        info!("Listening on {}", addr);

        let twitter_bearer_token = twitter_bearer_token.unwrap_or_else(String::new);
        let shared = Arc::new(SharedData {
            asset_proxy_endpoint,
            asset_proxy_count,
            twitter_bearer_token,
        });

        // TODO: db_ty indicates if any actions that mutate the database can be run
        let (db_pool, _db_ty) =
            db::connect(db::ConnectMode::Read).context("Failed to connect to Postgres")?;
        let db_pool = Arc::new(db_pool);

        let version_extension = format!(
            "/v{}",
            percent_encoding::utf8_percent_encode(
                env!("CARGO_PKG_VERSION_MAJOR"),
                percent_encoding::NON_ALPHANUMERIC,
            )
        );

        // Should look something like "/..."
        let graphiql_uri = version_extension.clone();
        assert!(graphiql_uri.starts_with('/'));

        let schema = Arc::new(schema::create());

        actix_web::rt::System::new()
            .block_on(
                HttpServer::new(move || {
                    App::new()
                        .app_data(actix_web::web::Data::new(schema.clone()))
                        .wrap(middleware::Logger::default())
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
                            web::resource(&version_extension)
                                .route(web::post().to(graphql(db_pool.clone(), shared.clone()))),
                        )
                        .service(
                            web::resource("/graphiql")
                                .route(web::get().to(graphiql(graphiql_uri.clone()))),
                        )
                })
                .bind(addr)?
                .run(),
            )
            .context("Actix server failed to run")
    });
}
