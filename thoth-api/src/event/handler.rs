use super::model::{Event, EventType};
use crate::model::work::{Work, WorkProperties};
use crate::redis::{lpush, RedisPool};
use thoth_errors::ThothResult;

pub const QUEUE_KEY: &str = "events:graphql";

pub async fn send_event(
    redis: &RedisPool,
    event_type: EventType,
    work: &Work,
) -> ThothResult<String> {
    let event = Event {
        event_type,
        work_id: *work.work_id(),
        is_published: work.is_active_withdrawn_superseded(),
        event_timestamp: work.updated_at,
    };
    lpush(redis, QUEUE_KEY, &serde_json::to_string(&event)?).await
}
