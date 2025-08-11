use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::affiliation::AffiliationWithInstitution;
use crate::model::work::WorkWithRelations;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::contribution;
#[cfg(feature = "backend")]
use crate::schema::contribution_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Role describing the type of contribution to the work"),
    ExistingTypePath = "crate::schema::sql_types::ContributionType"
)]
#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum ContributionType {
    #[cfg_attr(feature = "backend", graphql(description = "Author of the work"))]
    #[default]
    Author,
    #[cfg_attr(feature = "backend", graphql(description = "Editor of the work"))]
    Editor,
    #[cfg_attr(feature = "backend", graphql(description = "Translator of the work"))]
    Translator,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Photographer when named as the primary creator of, eg, a book of photographs"
        )
    )]
    Photographer,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Artist when named as the creator of artwork which illustrates a work"
        )
    )]
    Illustrator,
    #[cfg_attr(
        feature = "backend",
        db_rename = "music-editor",
        graphql(
            description = "Person responsible for editing any piece of music referenced in the work"
        )
    )]
    MusicEditor,
    #[cfg_attr(
        feature = "backend",
        db_rename = "foreword-by",
        graphql(description = "Author of foreword")
    )]
    ForewordBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "introduction-by",
        graphql(description = "Author of introduction")
    )]
    IntroductionBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "afterword-by",
        graphql(description = "Author of afterword")
    )]
    AfterwordBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "preface-by",
        graphql(description = "Author of preface")
    )]
    PrefaceBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "software-by",
        graphql(description = "Writer of computer programs ancillary to the work")
    )]
    SoftwareBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "research-by",
        graphql(
            description = "Person responsible for performing research on which the work is based"
        )
    )]
    ResearchBy,
    #[cfg_attr(
        feature = "backend",
        db_rename = "contributions-by",
        graphql(description = "Author of additional contributions to the work")
    )]
    ContributionsBy,
    #[cfg_attr(feature = "backend", graphql(description = "Compiler of index"))]
    Indexer,
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
    CreatedAt,
    UpdatedAt,
    FirstName,
    LastName,
    FullName,
    ContributionOrdinal,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    // pub biography: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionWithAffiliations {
    pub affiliations: Option<Vec<AffiliationWithInstitution>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionWithWork {
    pub work: WorkWithRelations,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new individual involvement in the production of a work"),
    diesel(table_name = contribution)
)]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    // pub biography: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an individual involvement in the production of a work"),
    diesel(table_name = contribution, treat_none_as_null = true)
)]
pub struct PatchContribution {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    // pub biography: Option<String>,
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
    diesel(table_name = contribution_history)
)]
pub struct NewContributionHistory {
    pub contribution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl Default for Contribution {
    fn default() -> Contribution {
        Contribution {
            contribution_id: Default::default(),
            work_id: Default::default(),
            contributor_id: Default::default(),
            contribution_type: Default::default(),
            main_contribution: true,
            // biography: Default::default(),
            created_at: Default::default(),
            updated_at: Default::default(),
            first_name: Default::default(),
            last_name: Default::default(),
            full_name: Default::default(),
            contribution_ordinal: 1,
        }
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
    assert_eq!(format!("{}", ContributionType::Illustrator), "Illustrator");
    assert_eq!(format!("{}", ContributionType::MusicEditor), "Music Editor");
    assert_eq!(format!("{}", ContributionType::ForewordBy), "Foreword By");
    assert_eq!(
        format!("{}", ContributionType::IntroductionBy),
        "Introduction By"
    );
    assert_eq!(format!("{}", ContributionType::AfterwordBy), "Afterword By");
    assert_eq!(format!("{}", ContributionType::PrefaceBy), "Preface By");
    assert_eq!(format!("{}", ContributionType::SoftwareBy), "Software By");
    assert_eq!(format!("{}", ContributionType::ResearchBy), "Research By");
    assert_eq!(
        format!("{}", ContributionType::ContributionsBy),
        "Contributions By"
    );
    assert_eq!(format!("{}", ContributionType::Indexer), "Indexer");
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
        ContributionType::from_str("Illustrator").unwrap(),
        ContributionType::Illustrator
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
    assert_eq!(
        ContributionType::from_str("Software By").unwrap(),
        ContributionType::SoftwareBy
    );
    assert_eq!(
        ContributionType::from_str("Research By").unwrap(),
        ContributionType::ResearchBy
    );
    assert_eq!(
        ContributionType::from_str("Contributions By").unwrap(),
        ContributionType::ContributionsBy
    );
    assert_eq!(
        ContributionType::from_str("Indexer").unwrap(),
        ContributionType::Indexer
    );

    assert!(ContributionType::from_str("Juggler").is_err());
    assert!(ContributionType::from_str("Supervisor").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
