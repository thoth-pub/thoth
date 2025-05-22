use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::event::model::EventType;
use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::webhook;
#[cfg(feature = "backend")]
use crate::schema::webhook_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting webhooks list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookField {
    WebhookId,
    PublisherId,
    #[default]
    Endpoint,
    Token,
    IsPublished,
    EventType,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub webhook_id: Uuid,
    pub publisher_id: Uuid,
    pub endpoint: String,
    pub token: Option<String>,
    pub is_published: bool,
    pub event_type: EventType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new webhook"),
    diesel(table_name = webhook)
)]
pub struct NewWebhook {
    pub publisher_id: Uuid,
    pub endpoint: String,
    pub token: Option<String>,
    pub is_published: bool,
    pub event_type: EventType,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing webhook"),
    diesel(table_name = webhook, treat_none_as_null = true)
)]
pub struct PatchWebhook {
    pub webhook_id: Uuid,
    pub publisher_id: Uuid,
    pub endpoint: String,
    pub token: Option<String>,
    pub is_published: bool,
    pub event_type: EventType,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct WebhookHistory {
    pub webhook_history_id: Uuid,
    pub webhook_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = webhook_history)
)]
pub struct NewWebhookHistory {
    pub webhook_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting webhooks list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookOrderBy {
    pub field: WebhookField,
    pub direction: Direction,
}

#[test]
fn test_webhookfield_default() {
    let webfield: WebhookField = Default::default();
    assert_eq!(webfield, WebhookField::Endpoint);
}

#[cfg(feature = "backend")]
pub mod crud;
