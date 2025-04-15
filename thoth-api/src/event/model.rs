use crate::model::Timestamp;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, juniper::GraphQLEnum)]
pub enum EventType {
    WorkCreated,
    WorkUpdated,
    WorkPublished,
}

#[derive(Serialize, juniper::GraphQLInputObject)]
pub struct Event {
    pub event_type: EventType,
    pub work_id: Uuid,
    pub is_published: bool,
    pub event_timestamp: Timestamp,
}
