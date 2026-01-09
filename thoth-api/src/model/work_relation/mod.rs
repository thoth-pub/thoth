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

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Nature of a relationship between works"),
    ExistingTypePath = "crate::schema::sql_types::RelationType"
)]
#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum RelationType {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work to which this relation belongs replaces the other work in the relationship"
        )
    )]
    Replaces,
    #[cfg_attr(
        feature = "backend",
        db_rename = "has-translation",
        graphql(
            description = "The work to which this relation belongs is translated by the other work in the relationship"
        )
    )]
    HasTranslation,
    #[cfg_attr(
        feature = "backend",
        db_rename = "has-part",
        graphql(
            description = "The work to which this relation belongs contains the other work (part) in the relationship"
        )
    )]
    HasPart,
    #[cfg_attr(
        feature = "backend",
        db_rename = "has-child",
        graphql(
            description = "The work to which this relation belongs contains the other work (chapter) in the relationship"
        )
    )]
    #[default]
    HasChild,
    #[cfg_attr(
        feature = "backend",
        db_rename = "is-replaced-by",
        graphql(
            description = "The work to which this relation belongs is replaced by the other work in the relationship"
        )
    )]
    IsReplacedBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "is-translation-of",
        graphql(
            description = "The work to which this relation belongs is a translation of the other work in the relationship"
        )
    )]
    IsTranslationOf,
    #[cfg_attr(
        feature = "backend",
        db_rename = "is-part-of",
        graphql(
            description = "The work to which this relation belongs is a component (part) of the other work in the relationship"
        )
    )]
    IsPartOf,
    #[cfg_attr(
        feature = "backend",
        db_rename = "is-child-of",
        graphql(
            description = "The work to which this relation belongs is a component (chapter) of the other work in the relationship"
        )
    )]
    IsChildOf,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting work relations list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkRelationField {
    WorkRelationId,
    RelatorWorkId,
    RelatedWorkId,
    #[default]
    RelationType,
    RelationOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    graphql(description = "Set of values required to define a new relationship between two works"),
    diesel(table_name = work_relation)
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
    graphql(description = "Set of values required to update an existing relationship between two works"),
    diesel(table_name = work_relation, treat_none_as_null = true)
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
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = work_relation_history)
)]
pub struct NewWorkRelationHistory {
    pub work_relation_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting work relations list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkRelationOrderBy {
    pub field: WorkRelationField,
    pub direction: Direction,
}

#[cfg(feature = "backend")]
impl RelationType {
    fn convert_to_inverse(&self) -> RelationType {
        match self {
            RelationType::Replaces => RelationType::IsReplacedBy,
            RelationType::HasTranslation => RelationType::IsTranslationOf,
            RelationType::HasPart => RelationType::IsPartOf,
            RelationType::HasChild => RelationType::IsChildOf,
            RelationType::IsReplacedBy => RelationType::Replaces,
            RelationType::IsTranslationOf => RelationType::HasTranslation,
            RelationType::IsPartOf => RelationType::HasPart,
            RelationType::IsChildOf => RelationType::HasChild,
        }
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
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::WorkRelationPolicy;
