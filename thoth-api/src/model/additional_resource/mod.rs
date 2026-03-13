use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::{Doi, Timestamp};
#[cfg(feature = "backend")]
use crate::schema::{additional_resource, additional_resource_history};

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of additional resource"),
    ExistingTypePath = "crate::schema::sql_types::ResourceType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceType {
    #[cfg_attr(feature = "backend", db_rename = "AUDIO")]
    Audio,
    #[cfg_attr(feature = "backend", db_rename = "VIDEO")]
    Video,
    #[cfg_attr(feature = "backend", db_rename = "IMAGE")]
    Image,
    #[cfg_attr(feature = "backend", db_rename = "BLOG")]
    Blog,
    #[cfg_attr(feature = "backend", db_rename = "WEBSITE")]
    Website,
    #[cfg_attr(feature = "backend", db_rename = "DOCUMENT")]
    Document,
    #[cfg_attr(feature = "backend", db_rename = "BOOK")]
    Book,
    #[cfg_attr(feature = "backend", db_rename = "ARTICLE")]
    Article,
    #[cfg_attr(feature = "backend", db_rename = "MAP")]
    Map,
    #[cfg_attr(feature = "backend", db_rename = "SOURCE")]
    Source,
    #[cfg_attr(feature = "backend", db_rename = "DATASET")]
    Dataset,
    #[cfg_attr(feature = "backend", db_rename = "SPREADSHEET")]
    Spreadsheet,
    #[default]
    #[cfg_attr(feature = "backend", db_rename = "OTHER")]
    Other,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting additional resources list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdditionalResourceField {
    AdditionalResourceId,
    WorkId,
    #[default]
    ResourceOrdinal,
    Title,
    Attribution,
    ResourceType,
    Doi,
    Handle,
    Url,
    Date,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalResource {
    pub additional_resource_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<Doi>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub date: Option<NaiveDate>,
    pub resource_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new additional resource linked to a work"),
    diesel(table_name = additional_resource)
)]
pub struct NewAdditionalResource {
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<Doi>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub date: Option<NaiveDate>,
    pub resource_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing additional resource"),
    diesel(table_name = additional_resource, treat_none_as_null = true)
)]
pub struct PatchAdditionalResource {
    pub additional_resource_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub attribution: Option<String>,
    pub resource_type: ResourceType,
    pub doi: Option<Doi>,
    pub handle: Option<String>,
    pub url: Option<String>,
    pub date: Option<NaiveDate>,
    pub resource_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct AdditionalResourceHistory {
    pub additional_resource_history_id: Uuid,
    pub additional_resource_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = additional_resource_history)
)]
pub struct NewAdditionalResourceHistory {
    pub additional_resource_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
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

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::AdditionalResourcePolicy;
#[cfg(test)]
mod tests;
