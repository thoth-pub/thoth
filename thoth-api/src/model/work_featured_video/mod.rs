use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::{work_featured_video, work_featured_video_history};

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkFeaturedVideo {
    pub work_featured_video_id: Uuid,
    pub work_id: Uuid,
    pub video_id: Option<String>,
    pub title: Option<String>,
    pub width: i32,
    pub height: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new featured video for a work"),
    diesel(table_name = work_featured_video)
)]
pub struct NewWorkFeaturedVideo {
    pub work_id: Uuid,
    pub video_id: Option<String>,
    pub title: Option<String>,
    pub width: i32,
    pub height: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing featured video"),
    diesel(table_name = work_featured_video, treat_none_as_null = true)
)]
pub struct PatchWorkFeaturedVideo {
    pub work_featured_video_id: Uuid,
    pub work_id: Uuid,
    pub video_id: Option<String>,
    pub title: Option<String>,
    pub width: i32,
    pub height: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = work_featured_video_history)
)]
pub struct NewWorkFeaturedVideoHistory {
    pub work_featured_video_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct WorkFeaturedVideoHistory {
    pub work_featured_video_history_id: Uuid,
    pub work_featured_video_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "backend")]
pub mod crud;
