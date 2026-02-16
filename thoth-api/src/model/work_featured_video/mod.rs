use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::work_featured_video;
#[cfg(feature = "backend")]
use crate::schema::work_featured_video_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting featured videos list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkFeaturedVideoField {
    WorkFeaturedVideoId,
    WorkId,
    Title,
    Url,
    Width,
    Height,
    CreatedAt,
    #[default]
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkFeaturedVideo {
    pub work_featured_video_id: Uuid,
    pub work_id: Uuid,
    pub title: Option<String>,
    pub url: Option<String>,
    pub width: i32,
    pub height: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new featured video linked to a work"),
    diesel(table_name = work_featured_video)
)]
pub struct NewWorkFeaturedVideo {
    pub work_id: Uuid,
    pub title: Option<String>,
    pub url: Option<String>,
    pub width: i32,
    pub height: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing featured video"),
    diesel(table_name = work_featured_video, treat_none_as_null = true)
)]
pub struct PatchWorkFeaturedVideo {
    pub work_featured_video_id: Uuid,
    pub work_id: Uuid,
    pub title: Option<String>,
    pub url: Option<String>,
    pub width: i32,
    pub height: i32,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct WorkFeaturedVideoHistory {
    pub work_featured_video_history_id: Uuid,
    pub work_featured_video_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = work_featured_video_history)
)]
pub struct NewWorkFeaturedVideoHistory {
    pub work_featured_video_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting featured videos list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkFeaturedVideoOrderBy {
    pub field: WorkFeaturedVideoField,
    pub direction: Direction,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::WorkFeaturedVideoPolicy;
#[cfg(test)]
mod tests;
