use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::{additional_resource, additional_resource_history};

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of additional resource"),
    ExistingTypePath = "crate::schema::sql_types::ResourceType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum ResourceType {
    Audio,
    Video,
    Image,
    Blog,
    Website,
    Document,
    Book,
    Article,
    Map,
    Source,
    Dataset,
    Spreadsheet,
    #[default]
    Other,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting additional resources list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdditionalResourceField {
    AdditionalResourceId,
    WorkId,
    #[default]
    ResourceOrdinal,
    Title,
    ResourceType,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalResource {
    pub additional_resource_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<String>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub resource_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new additional resource linked to a work"),
    diesel(table_name = additional_resource)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewAdditionalResource {
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<String>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub resource_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing additional resource"),
    diesel(table_name = additional_resource, treat_none_as_null = true)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchAdditionalResource {
    pub additional_resource_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<String>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub resource_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting additional resources list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdditionalResourceOrderBy {
    pub field: AdditionalResourceField,
    pub direction: Direction,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = additional_resource_history)
)]
pub struct NewAdditionalResourceHistory {
    pub additional_resource_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct AdditionalResourceHistory {
    pub additional_resource_history_id: Uuid,
    pub additional_resource_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "backend")]
pub mod crud;
