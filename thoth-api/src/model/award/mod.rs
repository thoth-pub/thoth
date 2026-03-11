use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::award;
#[cfg(feature = "backend")]
use crate::schema::award_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting awards list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AwardField {
    AwardId,
    WorkId,
    #[default]
    AwardOrdinal,
    Title,
    Category,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Award {
    pub award_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub note: Option<String>,
    pub award_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new award linked to a work"),
    diesel(table_name = award)
)]
pub struct NewAward {
    pub work_id: Uuid,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub note: Option<String>,
    pub award_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing award"),
    diesel(table_name = award, treat_none_as_null = true)
)]
pub struct PatchAward {
    pub award_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub note: Option<String>,
    pub award_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct AwardHistory {
    pub award_history_id: Uuid,
    pub award_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(diesel::Insertable), diesel(table_name = award_history))]
pub struct NewAwardHistory {
    pub award_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting awards list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AwardOrderBy {
    pub field: AwardField,
    pub direction: Direction,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::AwardPolicy;
#[cfg(test)]
mod tests;
