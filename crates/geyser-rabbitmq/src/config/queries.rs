use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/config/config-query.graphql",
    schema_path = "src/config/config-schema.graphql"
)]
struct Test;
