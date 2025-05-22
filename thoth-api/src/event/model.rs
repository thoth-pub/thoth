use crate::model::Timestamp;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Nature of an event"),
    ExistingTypePath = "crate::schema::sql_types::EventType"
)]
#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum EventType {
    #[cfg_attr(feature = "backend", graphql(description = "Work creation event"))]
    WorkCreated,
    #[default]
    #[cfg_attr(feature = "backend", graphql(description = "Work update event"))]
    WorkUpdated,
    #[cfg_attr(feature = "backend", graphql(description = "Work publication event"))]
    WorkPublished,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub event_type: EventType,
    pub work_id: Uuid,
    pub is_published: bool,
    pub event_timestamp: Timestamp,
    pub thoth_version: String,
}
