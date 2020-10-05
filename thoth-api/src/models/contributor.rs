use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::contribution;
#[cfg(feature = "backend")]
use crate::schema::contributor;
use crate::errors::ThothError;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Contribution_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "contributor")]
pub struct NewContributor {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Contribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "contribution")]
pub struct NewContribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}

impl fmt::Display for ContributionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContributionType::Author => write!(f, "Author"),
            ContributionType::Editor => write!(f, "Editor"),
            ContributionType::Translator => write!(f, "Translator"),
            ContributionType::Photographer => write!(f, "Photographer"),
            ContributionType::Ilustrator => write!(f, "Ilustrator"),
            ContributionType::MusicEditor => write!(f, "Music Editor"),
            ContributionType::ForewordBy => write!(f, "Foreword By"),
            ContributionType::IntroductionBy => write!(f, "Introduction By"),
            ContributionType::AfterwordBy => write!(f, "Afterword By"),
            ContributionType::PrefaceBy => write!(f, "Preface By"),
        }
    }
}

impl FromStr for ContributionType {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<ContributionType, ThothError> {
        match input {
            "Author" => Ok(ContributionType::Author),
            "Editor" => Ok(ContributionType::Editor),
            "Translator" => Ok(ContributionType::Translator),
            "Photographer" => Ok(ContributionType::Photographer),
            "Ilustrator" => Ok(ContributionType::Ilustrator),
            "Music Editor" => Ok(ContributionType::MusicEditor),
            "Foreword By" => Ok(ContributionType::ForewordBy),
            "Introduction By" => Ok(ContributionType::IntroductionBy),
            "Afterword By" => Ok(ContributionType::AfterwordBy),
            "Preface By" => Ok(ContributionType::PrefaceBy),
            _  => Err(ThothError::InvalidContributionType(input.to_string())),
        }
    }
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
    assert_eq!(ContributionType::from_str("Author").unwrap(), ContributionType::Author);
    assert_eq!(ContributionType::from_str("Editor").unwrap(), ContributionType::Editor);
    assert_eq!(ContributionType::from_str("Translator").unwrap(), ContributionType::Translator);
    assert_eq!(ContributionType::from_str("Photographer").unwrap(), ContributionType::Photographer);
    assert_eq!(ContributionType::from_str("Ilustrator").unwrap(), ContributionType::Ilustrator);
    assert_eq!(ContributionType::from_str("Music Editor").unwrap(), ContributionType::MusicEditor);
    assert_eq!(ContributionType::from_str("Foreword By").unwrap(), ContributionType::ForewordBy);
    assert_eq!(ContributionType::from_str("Introduction By").unwrap(), ContributionType::IntroductionBy);
    assert_eq!(ContributionType::from_str("Afterword By").unwrap(), ContributionType::AfterwordBy);
    assert_eq!(ContributionType::from_str("Preface By").unwrap(), ContributionType::PrefaceBy);

    assert!(ContributionType::from_str("Juggler").is_err());
    assert!(ContributionType::from_str("Supervisor").is_err());
}
