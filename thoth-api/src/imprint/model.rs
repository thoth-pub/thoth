use chrono::naive::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
use crate::graphql::utils::Direction;
#[cfg(feature = "backend")]
use crate::schema::imprint;
#[cfg(feature = "backend")]
use crate::schema::imprint_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting imprints list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImprintField {
    #[serde(rename = "IMPRINT_ID")]
    ImprintID,
    ImprintName,
    #[serde(rename = "IMPRINT_URL")]
    ImprintURL,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Imprint {
    pub imprint_id: Uuid,
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "imprint"
)]
pub struct NewImprint {
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "imprint"
)]
pub struct PatchImprint {
    pub imprint_id: Uuid,
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ImprintHistory {
    pub imprint_history_id: Uuid,
    pub imprint_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "imprint_history"
)]
pub struct NewImprintHistory {
    pub imprint_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting imprints list")
)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImprintOrderBy {
    pub field: ImprintField,
    pub direction: Direction,
}

impl Default for ImprintField {
    fn default() -> Self {
        ImprintField::ImprintName
    }
}

impl FromStr for ImprintField {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<ImprintField, ThothError> {
        match input {
            // Only match the headers which are currently defined/sortable in the UI
            "ID" => Ok(ImprintField::ImprintID),
            "Imprint" => Ok(ImprintField::ImprintName),
            "ImprintURL" => Ok(ImprintField::ImprintURL),
            "Updated" => Ok(ImprintField::UpdatedAt),
            _ => Err(ThothError::SortFieldError(
                input.to_string(),
                "Imprint".to_string(),
            )),
        }
    }
}

#[test]
fn test_imprintfield_default() {
    let impfield: ImprintField = Default::default();
    assert_eq!(impfield, ImprintField::ImprintName);
}

#[test]
fn test_imprintfield_fromstr() {
    assert_eq!(
        ImprintField::from_str("ID").unwrap(),
        ImprintField::ImprintID
    );
    assert_eq!(
        ImprintField::from_str("Imprint").unwrap(),
        ImprintField::ImprintName
    );
    assert_eq!(
        ImprintField::from_str("ImprintURL").unwrap(),
        ImprintField::ImprintURL
    );
    assert_eq!(
        ImprintField::from_str("Updated").unwrap(),
        ImprintField::UpdatedAt
    );
    assert!(ImprintField::from_str("Publisher").is_err());
    assert!(ImprintField::from_str("Created").is_err());
}
