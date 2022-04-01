pub type Schema = async_graphql::Schema<Query, Mutation, Subscription>;

pub struct Query;
pub struct Mutation;
pub struct Subscription;

pub struct SchemaData {}

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
        futures::stream::once(async { 3 })
    }
}

pub fn build(data: SchemaData) -> Schema {
    Schema::build(Query, Mutation, Subscription)
        .data(data)
        .finish()
}
