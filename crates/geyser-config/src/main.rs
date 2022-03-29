//! GraphQL server for remotely managing `holaplex-indexer` Geyser configurations

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};

struct Query;
struct Mutation;
struct Subscription;

type Schema = async_graphql::Schema<Query, Mutation, Subscription>;

#[async_graphql::Object]
impl Query {
    async fn test(&self) -> i32 {
        1
    }
}

#[async_graphql::Object]
impl Mutation {
    async fn test(&self) -> i32 {
        2
    }
}

#[async_graphql::Subscription]
impl Subscription {
    async fn test(&self) -> impl futures::stream::Stream<Item = i32> {
        futures::stream::once(async { 2 })
    }
}

struct Data {
    schema: Schema,
    object_route: &'static str,
    sub_route: &'static str,
}

struct SchemaData {}

async fn graphql(data: web::Data<Data>, req: GraphQLRequest) -> GraphQLResponse {
    data.schema.execute(req.into_inner()).await.into()
}

#[allow(clippy::unused_async)]
async fn graphql_ws(
    data: web::Data<Data>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&data.schema)).start(&req, payload)
}

#[allow(clippy::unused_async)]
async fn playground(data: web::Data<Data>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new(data.object_route).subscription_endpoint(data.sub_route),
        ))
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(Query, Mutation, Subscription)
        .data(SchemaData {})
        .finish();

    let data = web::Data::new(Data {
        schema,
        object_route: "/",
        sub_route: "/",
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::resource(data.object_route)
                    .guard(guard::Post())
                    .to(graphql),
            )
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(graphql_ws),
            )
            .service(web::resource("/").guard(guard::Get()).to(playground))
    })
    .bind("[::]:3000")?
    .run()
    .await
}
