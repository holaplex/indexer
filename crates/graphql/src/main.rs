//! GraphQL server to read from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{net::SocketAddr, sync::Arc};

use actix_cors::Cors;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use indexer_core::{clap, clap::Parser, db, ServerOpts};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use indexer_core::prelude::*;

use crate::schema::{Schema,AppContext};

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

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let db_pool =
    db::connect(db::ConnectMode::Read).context("Failed to connect to Postgres").unwrap();

    let db_connection = db_pool.get().unwrap();

    let ctx = AppContext::new(db_connection);
    let res = data.execute(&st, &ctx).await;

    let json = serde_json::to_string(&res)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json))
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            server: ServerOpts { port },
        } = Opts::parse();

        let db_conn =
            db::connect(db::ConnectMode::Read).context("Failed to connect to Postgres")?;

        let mut addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
        addr.set_port(port);
        info!("Listening on {}", addr);

        let graphiql_uri = format!("http://{}", addr);

        let schema = std::sync::Arc::new(schema::create(db_conn));

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
                        .service(web::resource("/").route(web::post().to(graphql)))
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
