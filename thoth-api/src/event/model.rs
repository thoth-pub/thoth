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

#[cfg_attr(feature = "backend", derive(Queryable, juniper::GraphQLObject))]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub webhook_id: Uuid,
    pub endpoint: String,
    pub token: Option<String>,
    pub is_published: bool,
    pub event_type: EventType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg(feature = "backend")]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use thoth_errors::ThothResult;
impl Webhook {
    pub fn all(
        db: &crate::db::PgPool,
        limit: i32,
        offset: i32,
        publisher_id: Option<Uuid>,
        event_types: Vec<EventType>,
        published: Option<bool>,
    ) -> ThothResult<Vec<Webhook>> {
        use crate::schema::webhook::dsl::*;
        let mut connection = db.get()?;
        let mut query = webhook
            .inner_join(crate::schema::publisher_webhook::table)
            .select(crate::schema::webhook::all_columns)
            .into_boxed();

        if let Some(pid) = publisher_id {
            query = query.filter(crate::schema::publisher_webhook::publisher_id.eq(pid));
        }
        if !event_types.is_empty() {
            query = query.filter(event_type.eq_any(event_types));
        }
        if let Some(boolean) = published {
            query = query.filter(is_published.eq(boolean));
        }
        query
            .limit(limit.into())
            .offset(offset.into())
            .load::<Webhook>(&mut connection)
            .map_err(Into::into)
    }
}
