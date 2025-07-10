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
    #[cfg_attr(
        feature = "backend",
        db_rename = "WorkCreated",
        graphql(description = "Work creation event")
    )]
    WorkCreated,
    #[default]
    #[cfg_attr(
        feature = "backend",
        db_rename = "WorkUpdated",
        graphql(description = "Work update event")
    )]
    WorkUpdated,
    #[cfg_attr(
        feature = "backend",
        db_rename = "WorkPublished",
        graphql(description = "Work publication event")
    )]
    WorkPublished,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "Details of a change made to a record which may require follow-up processing"
    )
)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub event_type: EventType,
    pub work_id: Uuid,
    pub is_published: bool,
    pub event_timestamp: Timestamp,
    pub thoth_version: String,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLObject))]
#[derive(Debug, Deserialize, Serialize)]
pub struct EventWrapper {
    pub event: Event,
}
