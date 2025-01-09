use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::event;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    ExistingTypePath = "crate::schema::sql_types::EventType"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    #[cfg_attr(feature = "backend", db_rename = "work-created")]
    WorkCreated,
    #[cfg_attr(feature = "backend", db_rename = "work-updated")]
    WorkUpdated,
    #[cfg_attr(feature = "backend", db_rename = "work-published")]
    WorkPublished,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Event {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub work_id: Uuid,
    pub is_published: bool,
    pub event_timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = event)
)]
pub struct NewEvent {
    pub event_type: EventType,
    pub work_id: Uuid,
    pub is_published: bool,
    pub event_timestamp: Timestamp,
}
