use serde_json::Value;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct CollectionDocument {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
    pub magic_eden_id: Option<String>,
    pub verified_collection_address: Option<String>,
    pub twitter_url: Option<String>,
    pub discord_url: Option<String>,
    pub website_url: Option<String>,
}

impl From<serde_json::Value> for CollectionDocument {
    fn from(value: serde_json::Value) -> Self {
        Self {
            id: value
                .get("id")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            name: value
                .get("name")
                .and_then(Value::as_str)
                .map(Into::into)
                .unwrap_or_default(),
            image: value.get("image").and_then(Value::as_str).map(Into::into),
            magic_eden_id: value
                .get("magic_eden_id")
                .and_then(Value::as_str)
                .map(Into::into),
            verified_collection_address: value
                .get("verified_collection_address")
                .and_then(Value::as_str)
                .map(Into::into),
            twitter_url: value
                .get("twitter_url")
                .and_then(Value::as_str)
                .map(Into::into),
            discord_url: value
                .get("discord_url")
                .and_then(Value::as_str)
                .map(Into::into),
            website_url: value
                .get("website_url")
                .and_then(Value::as_str)
                .map(Into::into),
        }
    }
}
