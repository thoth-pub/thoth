use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::contribution;
#[cfg(feature = "backend")]
use crate::schema::contribution_history;
use crate::work::model::WorkWithRelations;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Contribution_type")]
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum ContributionType {
    Author,
    Editor,
    Translator,
    Photographer,
    Ilustrator,
    #[cfg_attr(feature = "backend", db_rename = "music-editor")]
    MusicEditor,
    #[cfg_attr(feature = "backend", db_rename = "foreword-by")]
    ForewordBy,
    #[cfg_attr(feature = "backend", db_rename = "introduction-by")]
    IntroductionBy,
    #[cfg_attr(feature = "backend", db_rename = "afterword-by")]
    AfterwordBy,
    #[cfg_attr(feature = "backend", db_rename = "preface-by")]
    PrefaceBy,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting contributions list")
)]
pub enum ContributionField {
    ContributionId,
    WorkId,
    ContributorId,
    ContributionType,
    MainContribution,
    Biography,
    Institution,
    CreatedAt,
    UpdatedAt,
    FirstName,
    LastName,
    FullName,
    ContributionOrdinal,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionWithWork {
    pub work: WorkWithRelations,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "contribution"
)]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "contribution"
)]
pub struct PatchContribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ContributionHistory {
    pub contribution_history_id: Uuid,
    pub contribution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "contribution_history"
)]
pub struct NewContributionHistory {
    pub contribution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for ContributionType {
    fn default() -> ContributionType {
        ContributionType::Author
    }
}

#[test]
fn test_contributiontype_default() {
    let contributiontype: ContributionType = Default::default();
    assert_eq!(contributiontype, ContributionType::Author);
}

#[test]
fn test_contributiontype_display() {
    assert_eq!(format!("{}", ContributionType::Author), "Author");
    assert_eq!(format!("{}", ContributionType::Editor), "Editor");
    assert_eq!(format!("{}", ContributionType::Translator), "Translator");
    assert_eq!(
        format!("{}", ContributionType::Photographer),
        "Photographer"
    );
    assert_eq!(format!("{}", ContributionType::Ilustrator), "Ilustrator");
    assert_eq!(format!("{}", ContributionType::MusicEditor), "Music Editor");
    assert_eq!(format!("{}", ContributionType::ForewordBy), "Foreword By");
    assert_eq!(
        format!("{}", ContributionType::IntroductionBy),
        "Introduction By"
    );
    assert_eq!(format!("{}", ContributionType::AfterwordBy), "Afterword By");
    assert_eq!(format!("{}", ContributionType::PrefaceBy), "Preface By");
}

#[test]
fn test_contributiontype_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        ContributionType::from_str("Author").unwrap(),
        ContributionType::Author
    );
    assert_eq!(
        ContributionType::from_str("Editor").unwrap(),
        ContributionType::Editor
    );
    assert_eq!(
        ContributionType::from_str("Translator").unwrap(),
        ContributionType::Translator
    );
    assert_eq!(
        ContributionType::from_str("Photographer").unwrap(),
        ContributionType::Photographer
    );
    assert_eq!(
        ContributionType::from_str("Ilustrator").unwrap(),
        ContributionType::Ilustrator
    );
    assert_eq!(
        ContributionType::from_str("Music Editor").unwrap(),
        ContributionType::MusicEditor
    );
    assert_eq!(
        ContributionType::from_str("Foreword By").unwrap(),
        ContributionType::ForewordBy
    );
    assert_eq!(
        ContributionType::from_str("Introduction By").unwrap(),
        ContributionType::IntroductionBy
    );
    assert_eq!(
        ContributionType::from_str("Afterword By").unwrap(),
        ContributionType::AfterwordBy
    );
    assert_eq!(
        ContributionType::from_str("Preface By").unwrap(),
        ContributionType::PrefaceBy
    );

    assert!(ContributionType::from_str("Juggler").is_err());
    assert!(ContributionType::from_str("Supervisor").is_err());
}
