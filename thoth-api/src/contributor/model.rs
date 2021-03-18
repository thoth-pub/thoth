use chrono::naive::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContributorField {
    #[serde(rename = "CONTRIBUTOR_ID")]
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

impl FromStr for ContributorField {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<ContributorField, ThothError> {
        match input {
            // Only match the headers which are currently defined/sortable in the UI
            "ID" => Ok(ContributorField::ContributorID),
            "FullName" => Ok(ContributorField::FullName),
            "ORCID" => Ok(ContributorField::ORCID),
            "Updated" => Ok(ContributorField::UpdatedAt),
            _ => Err(ThothError::SortFieldError(
                input.to_string(),
                "Contributor".to_string(),
            )),
        }
    }
}

#[test]
fn test_contributorfield_fromstr() {
    assert_eq!(
        ContributorField::from_str("ID").unwrap(),
        ContributorField::ContributorID
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
        ContributorField::from_str("Updated").unwrap(),
        ContributorField::UpdatedAt
    );
    assert!(ContributorField::from_str("Website").is_err());
    assert!(ContributorField::from_str("Created").is_err());
}
