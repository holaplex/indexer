//! GraphQL server to read from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use actix_cors::Cors;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use indexer_core::{clap, clap::Parser, db, db::Pool, prelude::*, ServerOpts};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

use crate::schema::{AppContext, Schema};

mod schema;

#[derive(Parser)]
struct Opts {
    #[clap(flatten)]
    server: ServerOpts,
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
) -> impl Fn(
    web::Data<Arc<Schema>>,
    web::Json<GraphQLRequest>,
) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
+ Clone {
    move |st, data| {
        let pool = Arc::clone(&db_pool);

        Box::pin(async move {
            let ctx = AppContext::new(pool);
            let res = data.execute(&st, &ctx).await;

            let json = serde_json::to_string(&res)?;

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(json))
        })
    }
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            server: ServerOpts { port },
        } = Opts::parse();

        let db_pool =
            Arc::new(db::connect(db::ConnectMode::Read).context("Failed to connect to Postgres")?);

        let mut addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
        addr.set_port(port);
        info!("Listening on {}", addr);

        let graphiql_uri = format!("http://{}", addr);

        let schema = Arc::new(schema::create());

        actix_web::rt::System::new("main")
            .block_on(
                HttpServer::new(move || {
                    App::new()
                        .data(schema.clone())
                        .wrap(middleware::Logger::default())
                        .wrap(
                            Cors::new()
                                .allowed_methods(vec!["POST", "GET"])
                                .supports_credentials()
                                .max_age(3600)
                                .finish(),
                        )
                        .service(
                            web::resource(format!(
                                "/v{}",
                                percent_encoding::utf8_percent_encode(
                                    env!("CARGO_PKG_VERSION_MAJOR"),
                                    percent_encoding::NON_ALPHANUMERIC,
                                )
                            ))
                            .route(web::post().to(graphql(db_pool.clone()))),
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
