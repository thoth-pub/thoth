use chrono::naive::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::graphql::utils::Direction;
#[cfg(feature = "backend")]
use crate::schema::funder;
#[cfg(feature = "backend")]
use crate::schema::funder_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting funders list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunderField {
    #[serde(rename = "FUNDER_ID")]
    FunderID,
    FunderName,
    #[serde(rename = "FUNDER_DOI")]
    FunderDOI,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Funder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "funder"
)]
pub struct NewFunder {
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "funder"
)]
pub struct PatchFunder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct FunderHistory {
    pub funder_history_id: Uuid,
    pub funder_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(feature = "backend", derive(Insertable), table_name = "funder_history")]
pub struct NewFunderHistory {
    pub funder_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting funders list")
)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunderOrderBy {
    pub field: FunderField,
    pub direction: Direction,
}

impl Default for FunderField {
    fn default() -> Self {
        FunderField::FunderName
    }
}

impl FromStr for FunderField {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<FunderField, ThothError> {
        match input {
            // Only match the headers which are currently defined/sortable in the UI
            "ID" => Ok(FunderField::FunderID),
            "Funder" => Ok(FunderField::FunderName),
            "DOI" => Ok(FunderField::FunderDOI),
            "Updated" => Ok(FunderField::UpdatedAt),
            _ => Err(ThothError::SortFieldError(
                input.to_string(),
                "Funder".to_string(),
            )),
        }
    }
}

#[test]
fn test_funderfield_default() {
    let fundfield: FunderField = Default::default();
    assert_eq!(fundfield, FunderField::FunderName);
}

#[test]
fn test_funderfield_fromstr() {
    assert_eq!(FunderField::from_str("ID").unwrap(), FunderField::FunderID);
    assert_eq!(
        FunderField::from_str("Funder").unwrap(),
        FunderField::FunderName
    );
    assert_eq!(
        FunderField::from_str("DOI").unwrap(),
        FunderField::FunderDOI
    );
    assert_eq!(
        FunderField::from_str("Updated").unwrap(),
        FunderField::UpdatedAt
    );
    assert!(FunderField::from_str("Website").is_err());
    assert!(FunderField::from_str("Created").is_err());
}
