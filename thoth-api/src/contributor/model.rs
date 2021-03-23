use chrono::naive::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
#[cfg(feature = "backend")]
use crate::schema::contributor;
#[cfg(feature = "backend")]
use crate::schema::contributor_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting contributors list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContributorField {
    #[serde(rename = "CONTRIBUTOR_ID")]
    #[strum(serialize = "ID")]
    ContributorID,
    FirstName,
    LastName,
    FullName,
    #[serde(rename = "ORCID")]
    ORCID,
    Website,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "contributor"
)]
pub struct NewContributor {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "contributor"
)]
pub struct PatchContributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ContributorHistory {
    pub contributor_history_id: Uuid,
    pub contributor_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "contributor_history"
)]
pub struct NewContributorHistory {
    pub contributor_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting contributors list")
)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributorOrderBy {
    pub field: ContributorField,
    pub direction: Direction,
}

impl Default for ContributorField {
    fn default() -> Self {
        ContributorField::FullName
    }
}

#[test]
fn test_contributorfield_default() {
    let contfield: ContributorField = Default::default();
    assert_eq!(contfield, ContributorField::FullName);
}

#[test]
fn test_contributorfield_display() {
    assert_eq!(format!("{}", ContributorField::ContributorID), "ID");
    assert_eq!(format!("{}", ContributorField::FirstName), "FirstName");
    assert_eq!(format!("{}", ContributorField::LastName), "LastName");
    assert_eq!(format!("{}", ContributorField::FullName), "FullName");
    assert_eq!(format!("{}", ContributorField::ORCID), "ORCID");
    assert_eq!(format!("{}", ContributorField::Website), "Website");
    assert_eq!(format!("{}", ContributorField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", ContributorField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_contributorfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        ContributorField::from_str("ID").unwrap(),
        ContributorField::ContributorID
    );
    assert_eq!(
        ContributorField::from_str("FirstName").unwrap(),
        ContributorField::FirstName
    );
    assert_eq!(
        ContributorField::from_str("LastName").unwrap(),
        ContributorField::LastName
    );
    assert_eq!(
        ContributorField::from_str("FullName").unwrap(),
        ContributorField::FullName
    );
    assert_eq!(
        ContributorField::from_str("ORCID").unwrap(),
        ContributorField::ORCID
    );
    assert_eq!(
        ContributorField::from_str("UpdatedAt").unwrap(),
        ContributorField::UpdatedAt
    );
    assert!(ContributorField::from_str("ContributorID").is_err());
    assert!(ContributorField::from_str("Biography").is_err());
    assert!(ContributorField::from_str("Institution").is_err());
}
