use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::subject;
#[cfg(feature = "backend")]
use crate::schema::subject_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of a subject (e.g. the subject category scheme being used)"),
    ExistingTypePath = "crate::schema::sql_types::SubjectType"
)]
#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Deserialize,
    Serialize,
    EnumString,
    Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubjectType {
    #[strum(serialize = "BIC")]
    Bic,
    #[strum(serialize = "BISAC")]
    Bisac,
    Thema,
    #[strum(serialize = "LCC")]
    Lcc,
    Custom,
    #[default]
    Keyword,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting subjects list")
)]
pub enum SubjectField {
    SubjectId,
    WorkId,
    SubjectType,
    SubjectCode,
    SubjectOrdinal,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub subject_id: Uuid,
    pub work_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new significant discipline or term related to a work"),
    diesel(table_name = subject)
)]
pub struct NewSubject {
    pub work_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing significant discipline or term related to a work"),
    diesel(table_name = subject, treat_none_as_null = true)
)]
pub struct PatchSubject {
    pub subject_id: Uuid,
    pub work_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_code: String,
    pub subject_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct SubjectHistory {
    pub subject_history_id: Uuid,
    pub subject_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = subject_history)
)]
pub struct NewSubjectHistory {
    pub subject_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

impl Default for Subject {
    fn default() -> Subject {
        Subject {
            subject_id: Default::default(),
            work_id: Default::default(),
            subject_type: Default::default(),
            subject_code: Default::default(),
            subject_ordinal: 1,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

#[test]
fn test_subjecttype_default() {
    let subjecttype: SubjectType = Default::default();
    assert_eq!(subjecttype, SubjectType::Keyword);
}

#[test]
fn test_subjecttype_display() {
    assert_eq!(format!("{}", SubjectType::Bic), "BIC");
    assert_eq!(format!("{}", SubjectType::Bisac), "BISAC");
    assert_eq!(format!("{}", SubjectType::Thema), "Thema");
    assert_eq!(format!("{}", SubjectType::Lcc), "LCC");
    assert_eq!(format!("{}", SubjectType::Custom), "Custom");
    assert_eq!(format!("{}", SubjectType::Keyword), "Keyword");
}

#[test]
fn test_subjecttype_fromstr() {
    use std::str::FromStr;
    assert_eq!(SubjectType::from_str("BIC").unwrap(), SubjectType::Bic);
    assert_eq!(SubjectType::from_str("BISAC").unwrap(), SubjectType::Bisac);
    assert_eq!(SubjectType::from_str("Thema").unwrap(), SubjectType::Thema);
    assert_eq!(SubjectType::from_str("LCC").unwrap(), SubjectType::Lcc);
    assert_eq!(
        SubjectType::from_str("Custom").unwrap(),
        SubjectType::Custom
    );
    assert_eq!(
        SubjectType::from_str("Keyword").unwrap(),
        SubjectType::Keyword
    );

    assert!(SubjectType::from_str("bic").is_err());
    assert!(SubjectType::from_str("Library of Congress Subject Code").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
mod thema;
#[cfg(feature = "backend")]
pub(crate) use policy::SubjectPolicy;
