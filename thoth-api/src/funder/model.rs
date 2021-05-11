use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunderField {
    #[strum(serialize = "ID")]
    FunderId,
    #[strum(serialize = "Funder")]
    FunderName,
    #[strum(serialize = "DOI")]
    FunderDoi,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub timestamp: DateTime<Utc>,
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

impl Default for Funder {
    fn default() -> Funder {
        Funder {
            funder_id: Default::default(),
            funder_name: "".to_string(),
            funder_doi: None,
            created_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
        }
    }
}

impl fmt::Display for Funder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(doi) = &self.funder_doi {
            write!(f, "{} - {}", &self.funder_name, doi)
        } else {
            write!(f, "{}", &self.funder_name)
        }
    }
}

#[test]
fn test_funderfield_default() {
    let fundfield: FunderField = Default::default();
    assert_eq!(fundfield, FunderField::FunderName);
}

#[test]
fn test_funderfield_display() {
    assert_eq!(format!("{}", FunderField::FunderId), "ID");
    assert_eq!(format!("{}", FunderField::FunderName), "Funder");
    assert_eq!(format!("{}", FunderField::FunderDoi), "DOI");
    assert_eq!(format!("{}", FunderField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", FunderField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_funderfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(FunderField::from_str("ID").unwrap(), FunderField::FunderId);
    assert_eq!(
        FunderField::from_str("Funder").unwrap(),
        FunderField::FunderName
    );
    assert_eq!(
        FunderField::from_str("DOI").unwrap(),
        FunderField::FunderDoi
    );
    assert_eq!(
        FunderField::from_str("CreatedAt").unwrap(),
        FunderField::CreatedAt
    );
    assert_eq!(
        FunderField::from_str("UpdatedAt").unwrap(),
        FunderField::UpdatedAt
    );
    assert!(FunderField::from_str("FunderID").is_err());
    assert!(FunderField::from_str("Website").is_err());
    assert!(FunderField::from_str("Fundings").is_err());
}
