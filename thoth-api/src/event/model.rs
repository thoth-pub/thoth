use crate::model::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Nature of an event"),
    ExistingTypePath = "crate::schema::sql_types::EventType"
)]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum EventType {
    WorkCreated,
    WorkUpdated,
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
