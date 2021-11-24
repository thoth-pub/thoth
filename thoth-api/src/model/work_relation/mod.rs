use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::work_relation;
#[cfg(feature = "backend")]
use crate::schema::work_relation_history;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Relation_type")]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum RelationType {
    Replaces,
    #[cfg_attr(feature = "backend", db_rename = "has-translation")]
    HasTranslation,
    #[cfg_attr(feature = "backend", db_rename = "has-part")]
    HasPart,
    #[cfg_attr(feature = "backend", db_rename = "has-child")]
    HasChild,
    #[cfg_attr(feature = "backend", db_rename = "is-replaced-by")]
    IsReplacedBy,
    #[cfg_attr(feature = "backend", db_rename = "is-translation-of")]
    IsTranslationOf,
    #[cfg_attr(feature = "backend", db_rename = "is-part-of")]
    IsPartOf,
    #[cfg_attr(feature = "backend", db_rename = "is-child-of")]
    IsChildOf,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting work relations list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkRelationField {
    WorkRelationId,
    RelatorWorkId,
    RelatedWorkId,
    RelationType,
    RelationOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkRelation {
    pub work_relation_id: Uuid,
    pub relator_work_id: Uuid,
    pub related_work_id: Uuid,
    pub relation_type: RelationType,
    pub relation_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "work_relation"
)]
pub struct NewWorkRelation {
    pub relator_work_id: Uuid,
    pub related_work_id: Uuid,
    pub relation_type: RelationType,
    pub relation_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "work_relation"
)]
pub struct PatchWorkRelation {
    pub work_relation_id: Uuid,
    pub relator_work_id: Uuid,
    pub related_work_id: Uuid,
    pub relation_type: RelationType,
    pub relation_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct WorkRelationHistory {
    pub work_relation_history_id: Uuid,
    pub work_relation_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "work_relation_history"
)]
pub struct NewWorkRelationHistory {
    pub work_relation_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting work relations list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WorkRelationOrderBy {
    pub field: WorkRelationField,
    pub direction: Direction,
}

impl Default for RelationType {
    fn default() -> RelationType {
        RelationType::HasChild
    }
}

impl Default for WorkRelationField {
    fn default() -> Self {
        WorkRelationField::RelationType
    }
}

#[test]
fn test_relationtype_default() {
    let reltype: RelationType = Default::default();
    assert_eq!(reltype, RelationType::HasChild);
}

#[test]
fn test_workrelationfield_default() {
    let workrelfield: WorkRelationField = Default::default();
    assert_eq!(workrelfield, WorkRelationField::RelationType);
}

#[test]
fn test_relationtype_display() {
    assert_eq!(format!("{}", RelationType::Replaces), "Replaces");
    assert_eq!(
        format!("{}", RelationType::HasTranslation),
        "Has Translation"
    );
    assert_eq!(format!("{}", RelationType::HasPart), "Has Part");
    assert_eq!(format!("{}", RelationType::HasChild), "Has Child");
    assert_eq!(format!("{}", RelationType::IsReplacedBy), "Is Replaced By");
    assert_eq!(
        format!("{}", RelationType::IsTranslationOf),
        "Is Translation Of"
    );
    assert_eq!(format!("{}", RelationType::IsPartOf), "Is Part Of");
    assert_eq!(format!("{}", RelationType::IsChildOf), "Is Child Of");
}

#[test]
fn test_relationtype_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        RelationType::from_str("Replaces").unwrap(),
        RelationType::Replaces
    );
    assert_eq!(
        RelationType::from_str("Has Translation").unwrap(),
        RelationType::HasTranslation
    );
    assert_eq!(
        RelationType::from_str("Has Part").unwrap(),
        RelationType::HasPart
    );
    assert_eq!(
        RelationType::from_str("Has Child").unwrap(),
        RelationType::HasChild
    );
    assert_eq!(
        RelationType::from_str("Is Replaced By").unwrap(),
        RelationType::IsReplacedBy
    );
    assert_eq!(
        RelationType::from_str("Is Translation Of").unwrap(),
        RelationType::IsTranslationOf
    );
    assert_eq!(
        RelationType::from_str("Is Part Of").unwrap(),
        RelationType::IsPartOf
    );
    assert_eq!(
        RelationType::from_str("Is Child Of").unwrap(),
        RelationType::IsChildOf
    );

    assert!(RelationType::from_str("Has Parent").is_err());
    assert!(RelationType::from_str("Subsumes").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
