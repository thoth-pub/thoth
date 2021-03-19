use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::graphql::utils::Direction;
#[cfg(feature = "backend")]
use crate::schema::publisher;
#[cfg(feature = "backend")]
use crate::schema::publisher_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting publishers list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublisherField {
    #[serde(rename = "PUBLISHER_ID")]
    PublisherID,
    PublisherName,
    PublisherShortname,
    #[serde(rename = "PUBLISHER_URL")]
    PublisherURL,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "publisher"
)]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "publisher"
)]
pub struct PatchPublisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PublisherHistory {
    pub publisher_history_id: Uuid,
    pub publisher_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "publisher_history"
)]
pub struct NewPublisherHistory {
    pub publisher_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting publishers list")
)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublisherOrderBy {
    pub field: PublisherField,
    pub direction: Direction,
}

impl Default for PublisherField {
    fn default() -> Self {
        PublisherField::PublisherName
    }
}

impl fmt::Display for Publisher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.publisher_name)
    }
}

impl FromStr for PublisherField {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<PublisherField, ThothError> {
        match input {
            // Only match the headers which are currently defined/sortable in the UI
            "ID" => Ok(PublisherField::PublisherID),
            "Name" => Ok(PublisherField::PublisherName),
            "ShortName" => Ok(PublisherField::PublisherShortname),
            "URL" => Ok(PublisherField::PublisherURL),
            "Updated" => Ok(PublisherField::UpdatedAt),
            _ => Err(ThothError::SortFieldError(
                input.to_string(),
                "Publisher".to_string(),
            )),
        }
    }
}

#[test]
fn test_publisherfield_default() {
    let pubfield: PublisherField = Default::default();
    assert_eq!(pubfield, PublisherField::PublisherName);
}

#[test]
fn test_publisherfield_fromstr() {
    assert_eq!(
        PublisherField::from_str("ID").unwrap(),
        PublisherField::PublisherID
    );
    assert_eq!(
        PublisherField::from_str("Name").unwrap(),
        PublisherField::PublisherName
    );
    assert_eq!(
        PublisherField::from_str("ShortName").unwrap(),
        PublisherField::PublisherShortname
    );
    assert_eq!(
        PublisherField::from_str("URL").unwrap(),
        PublisherField::PublisherURL
    );
    assert_eq!(
        PublisherField::from_str("Updated").unwrap(),
        PublisherField::UpdatedAt
    );
    assert!(PublisherField::from_str("Website").is_err());
    assert!(PublisherField::from_str("Created").is_err());
}
