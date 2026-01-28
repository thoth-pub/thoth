use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::{endorsement, endorsement_history};

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting endorsements list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndorsementField {
    EndorsementId,
    WorkId,
    #[default]
    EndorsementOrdinal,
    AuthorName,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Endorsement {
    pub endorsement_id: Uuid,
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new endorsement linked to a work"),
    diesel(table_name = endorsement)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewEndorsement {
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing endorsement"),
    diesel(table_name = endorsement, treat_none_as_null = true)
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchEndorsement {
    pub endorsement_id: Uuid,
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting endorsements list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndorsementOrderBy {
    pub field: EndorsementField,
    pub direction: Direction,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = endorsement_history)
)]
pub struct NewEndorsementHistory {
    pub endorsement_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct EndorsementHistory {
    pub endorsement_history_id: Uuid,
    pub endorsement_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "backend")]
pub mod crud;

#[test]
fn test_endorsementfield_default() {
    let field: EndorsementField = Default::default();
    assert_eq!(field, EndorsementField::EndorsementOrdinal);
}

#[test]
fn test_endorsementfield_display() {
    assert_eq!(
        format!("{}", EndorsementField::EndorsementId),
        "EndorsementId"
    );
    assert_eq!(format!("{}", EndorsementField::WorkId), "WorkId");
    assert_eq!(
        format!("{}", EndorsementField::EndorsementOrdinal),
        "EndorsementOrdinal"
    );
    assert_eq!(format!("{}", EndorsementField::AuthorName), "AuthorName");
    assert_eq!(format!("{}", EndorsementField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", EndorsementField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_endorsementfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        EndorsementField::from_str("EndorsementId").unwrap(),
        EndorsementField::EndorsementId
    );
    assert_eq!(
        EndorsementField::from_str("WorkId").unwrap(),
        EndorsementField::WorkId
    );
    assert_eq!(
        EndorsementField::from_str("EndorsementOrdinal").unwrap(),
        EndorsementField::EndorsementOrdinal
    );
    assert_eq!(
        EndorsementField::from_str("AuthorName").unwrap(),
        EndorsementField::AuthorName
    );
    assert_eq!(
        EndorsementField::from_str("CreatedAt").unwrap(),
        EndorsementField::CreatedAt
    );
    assert_eq!(
        EndorsementField::from_str("UpdatedAt").unwrap(),
        EndorsementField::UpdatedAt
    );

    assert!(EndorsementField::from_str("endorsement_id").is_err());
}
