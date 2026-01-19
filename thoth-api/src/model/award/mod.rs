use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::{award, award_history};

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting awards list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AwardField {
    AwardId,
    WorkId,
    #[default]
    AwardOrdinal,
    Title,
    Category,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
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
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new award linked to a work"),
    diesel(table_name = award)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing award"),
    diesel(table_name = award, treat_none_as_null = true)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchAward {
    pub award_id: Uuid,
    pub work_id: Uuid,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub note: Option<String>,
    pub award_ordinal: i32,
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

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = award_history)
)]
pub struct NewAwardHistory {
    pub award_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct AwardHistory {
    pub award_history_id: Uuid,
    pub award_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "backend")]
pub mod crud;

#[test]
fn test_awardfield_default() {
    let field: AwardField = Default::default();
    assert_eq!(field, AwardField::AwardOrdinal);
}

#[test]
fn test_awardfield_display() {
    assert_eq!(format!("{}", AwardField::AwardId), "AwardId");
    assert_eq!(format!("{}", AwardField::WorkId), "WorkId");
    assert_eq!(format!("{}", AwardField::AwardOrdinal), "AwardOrdinal");
    assert_eq!(format!("{}", AwardField::Title), "Title");
    assert_eq!(format!("{}", AwardField::Category), "Category");
    assert_eq!(format!("{}", AwardField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", AwardField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_awardfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        AwardField::from_str("AwardId").unwrap(),
        AwardField::AwardId
    );
    assert_eq!(AwardField::from_str("WorkId").unwrap(), AwardField::WorkId);
    assert_eq!(
        AwardField::from_str("AwardOrdinal").unwrap(),
        AwardField::AwardOrdinal
    );
    assert_eq!(AwardField::from_str("Title").unwrap(), AwardField::Title);
    assert_eq!(
        AwardField::from_str("Category").unwrap(),
        AwardField::Category
    );
    assert_eq!(
        AwardField::from_str("CreatedAt").unwrap(),
        AwardField::CreatedAt
    );
    assert_eq!(
        AwardField::from_str("UpdatedAt").unwrap(),
        AwardField::UpdatedAt
    );

    assert!(AwardField::from_str("award_id").is_err());
}
