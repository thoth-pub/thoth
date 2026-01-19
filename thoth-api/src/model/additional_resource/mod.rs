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

#[test]
fn test_resourcetype_default() {
    let resource_type: ResourceType = Default::default();
    assert_eq!(resource_type, ResourceType::Other);
}

#[test]
fn test_resourcetype_display() {
    assert_eq!(format!("{}", ResourceType::Audio), "AUDIO");
    assert_eq!(format!("{}", ResourceType::Video), "VIDEO");
    assert_eq!(format!("{}", ResourceType::Image), "IMAGE");
    assert_eq!(format!("{}", ResourceType::Blog), "BLOG");
    assert_eq!(format!("{}", ResourceType::Website), "WEBSITE");
    assert_eq!(format!("{}", ResourceType::Document), "DOCUMENT");
    assert_eq!(format!("{}", ResourceType::Book), "BOOK");
    assert_eq!(format!("{}", ResourceType::Article), "ARTICLE");
    assert_eq!(format!("{}", ResourceType::Map), "MAP");
    assert_eq!(format!("{}", ResourceType::Source), "SOURCE");
    assert_eq!(format!("{}", ResourceType::Dataset), "DATASET");
    assert_eq!(format!("{}", ResourceType::Spreadsheet), "SPREADSHEET");
    assert_eq!(format!("{}", ResourceType::Other), "OTHER");
}

#[test]
fn test_resourcetype_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        ResourceType::from_str("AUDIO").unwrap(),
        ResourceType::Audio
    );
    assert_eq!(
        ResourceType::from_str("VIDEO").unwrap(),
        ResourceType::Video
    );
    assert_eq!(
        ResourceType::from_str("IMAGE").unwrap(),
        ResourceType::Image
    );
    assert_eq!(ResourceType::from_str("BLOG").unwrap(), ResourceType::Blog);
    assert_eq!(
        ResourceType::from_str("WEBSITE").unwrap(),
        ResourceType::Website
    );
    assert_eq!(
        ResourceType::from_str("DOCUMENT").unwrap(),
        ResourceType::Document
    );
    assert_eq!(ResourceType::from_str("BOOK").unwrap(), ResourceType::Book);
    assert_eq!(
        ResourceType::from_str("ARTICLE").unwrap(),
        ResourceType::Article
    );
    assert_eq!(ResourceType::from_str("MAP").unwrap(), ResourceType::Map);
    assert_eq!(
        ResourceType::from_str("SOURCE").unwrap(),
        ResourceType::Source
    );
    assert_eq!(
        ResourceType::from_str("DATASET").unwrap(),
        ResourceType::Dataset
    );
    assert_eq!(
        ResourceType::from_str("SPREADSHEET").unwrap(),
        ResourceType::Spreadsheet
    );
    assert_eq!(
        ResourceType::from_str("OTHER").unwrap(),
        ResourceType::Other
    );

    assert!(ResourceType::from_str("audio").is_err());
}

#[test]
fn test_additionalresourcefield_default() {
    let field: AdditionalResourceField = Default::default();
    assert_eq!(field, AdditionalResourceField::ResourceOrdinal);
}

#[test]
fn test_additionalresourcefield_display() {
    assert_eq!(
        format!("{}", AdditionalResourceField::AdditionalResourceId),
        "AdditionalResourceId"
    );
    assert_eq!(format!("{}", AdditionalResourceField::WorkId), "WorkId");
    assert_eq!(
        format!("{}", AdditionalResourceField::ResourceOrdinal),
        "ResourceOrdinal"
    );
    assert_eq!(format!("{}", AdditionalResourceField::Title), "Title");
    assert_eq!(
        format!("{}", AdditionalResourceField::ResourceType),
        "ResourceType"
    );
    assert_eq!(
        format!("{}", AdditionalResourceField::CreatedAt),
        "CreatedAt"
    );
    assert_eq!(
        format!("{}", AdditionalResourceField::UpdatedAt),
        "UpdatedAt"
    );
}

#[test]
fn test_additionalresourcefield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        AdditionalResourceField::from_str("AdditionalResourceId").unwrap(),
        AdditionalResourceField::AdditionalResourceId
    );
    assert_eq!(
        AdditionalResourceField::from_str("WorkId").unwrap(),
        AdditionalResourceField::WorkId
    );
    assert_eq!(
        AdditionalResourceField::from_str("ResourceOrdinal").unwrap(),
        AdditionalResourceField::ResourceOrdinal
    );
    assert_eq!(
        AdditionalResourceField::from_str("Title").unwrap(),
        AdditionalResourceField::Title
    );
    assert_eq!(
        AdditionalResourceField::from_str("ResourceType").unwrap(),
        AdditionalResourceField::ResourceType
    );
    assert_eq!(
        AdditionalResourceField::from_str("CreatedAt").unwrap(),
        AdditionalResourceField::CreatedAt
    );
    assert_eq!(
        AdditionalResourceField::from_str("UpdatedAt").unwrap(),
        AdditionalResourceField::UpdatedAt
    );

    assert!(AdditionalResourceField::from_str("additional_resource_id").is_err());
}
