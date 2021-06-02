use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::str::FromStr;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::errors::{ThothError, ThothResult};
use crate::graphql::utils::Direction;
use crate::model::Timestamp;
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
    #[strum(serialize = "ID")]
    ContributorId,
    FirstName,
    LastName,
    FullName,
    #[serde(rename = "ORCID")]
    #[strum(serialize = "ORCID")]
    Orcid,
    Website,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalarValue)
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Orcid(String);

pub const ORCID_DOMAIN: &str = "https://orcid.org/";

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<Orcid>,
    pub website: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
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
    pub orcid: Option<Orcid>,
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
    pub orcid: Option<Orcid>,
    pub website: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ContributorHistory {
    pub contributor_history_id: Uuid,
    pub contributor_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorOrderBy {
    pub field: ContributorField,
    pub direction: Direction,
}

impl Default for ContributorField {
    fn default() -> Self {
        ContributorField::FullName
    }
}

impl fmt::Display for Contributor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(orcid) = &self.orcid {
            write!(f, "{} - {}", &self.full_name, orcid)
        } else {
            write!(f, "{}", self.full_name)
        }
    }
}

impl Default for Orcid {
    fn default() -> Orcid {
        Orcid {
            0: Default::default(),
        }
    }
}

impl fmt::Display for Orcid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(ORCID_DOMAIN, ""))
    }
}

impl FromStr for Orcid {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Orcid> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
            // ^    = beginning of string
            // (?:) = non-capturing group
            // i    = case-insensitive flag
            // $    = end of string
            // Matches strings of format "[[http[s]://]orcid.org/]0000-000X-XXXX-XXXX"
            // and captures the 16-digit identifier segment
            // Corresponds to database constraints although regex syntax differs slightly
            r#"^(?i:(?:https?://)?orcid\.org/)?(0000-000(?:1-[5-9]|2-[0-9]|3-[0-4])\d{3}-\d{3}[\dX]$)"#).unwrap();
        }
        if let Some(matches) = RE.captures(input) {
            // The 0th capture always corresponds to the entire match
            if let Some(identifier) = matches.get(1) {
                let standardised = format!("{}{}", ORCID_DOMAIN, identifier.as_str());
                let orcid: Orcid = Orcid { 0: standardised };
                Ok(orcid)
            } else {
                Err(ThothError::IdentifierParseError(
                    input.to_string(),
                    "ORCID".to_string(),
                ))
            }
        } else {
            Err(ThothError::IdentifierParseError(
                input.to_string(),
                "ORCID".to_string(),
            ))
        }
    }
}

#[test]
fn test_contributorfield_default() {
    let contfield: ContributorField = Default::default();
    assert_eq!(contfield, ContributorField::FullName);
}

#[test]
fn test_contributorfield_display() {
    assert_eq!(format!("{}", ContributorField::ContributorId), "ID");
    assert_eq!(format!("{}", ContributorField::FirstName), "FirstName");
    assert_eq!(format!("{}", ContributorField::LastName), "LastName");
    assert_eq!(format!("{}", ContributorField::FullName), "FullName");
    assert_eq!(format!("{}", ContributorField::Orcid), "ORCID");
    assert_eq!(format!("{}", ContributorField::Website), "Website");
    assert_eq!(format!("{}", ContributorField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", ContributorField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_contributorfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        ContributorField::from_str("ID").unwrap(),
        ContributorField::ContributorId
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
        ContributorField::Orcid
    );
    assert_eq!(
        ContributorField::from_str("UpdatedAt").unwrap(),
        ContributorField::UpdatedAt
    );
    assert!(ContributorField::from_str("ContributorID").is_err());
    assert!(ContributorField::from_str("Biography").is_err());
    assert!(ContributorField::from_str("Institution").is_err());
}

#[test]
fn test_orcid_default() {
    let orcid: Orcid = Default::default();
    assert_eq!(orcid, Orcid { 0: "".to_string() });
}

#[test]
fn test_orcid_display() {
    let orcid = Orcid {
        0: "https://orcid.org/0000-0002-1234-5678".to_string(),
    };
    assert_eq!(format!("{}", orcid), "0000-0002-1234-5678");
}

#[test]
fn test_orcid_fromstr() {
    let standardised = Orcid {
        0: "https://orcid.org/0000-0002-1234-5678".to_string(),
    };
    assert_eq!(
        Orcid::from_str("https://orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("http://orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("orcid.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("HTTPS://ORCID.ORG/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert_eq!(
        Orcid::from_str("Https://ORCiD.org/0000-0002-1234-5678").unwrap(),
        standardised
    );
    assert!(Orcid::from_str("htts://orcid.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("http://test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("test.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("//orcid.org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("https://orcid-org/0000-0002-1234-5678").is_err());
    assert!(Orcid::from_str("0000-0002-1234-5678https://orcid.org/").is_err());
}
