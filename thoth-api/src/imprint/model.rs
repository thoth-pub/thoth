use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImprintField {
    #[serde(rename = "IMPRINT_ID")]
    #[strum(serialize = "ID")]
    ImprintID,
    #[strum(serialize = "Imprint")]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub timestamp: DateTime<Utc>,
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

#[test]
fn test_imprintfield_default() {
    let impfield: ImprintField = Default::default();
    assert_eq!(impfield, ImprintField::ImprintName);
}

#[test]
fn test_imprintfield_display() {
    assert_eq!(format!("{}", ImprintField::ImprintID), "ID");
    assert_eq!(format!("{}", ImprintField::ImprintName), "Imprint");
    assert_eq!(format!("{}", ImprintField::ImprintURL), "ImprintURL");
    assert_eq!(format!("{}", ImprintField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", ImprintField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_imprintfield_fromstr() {
    use std::str::FromStr;
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
        ImprintField::from_str("CreatedAt").unwrap(),
        ImprintField::CreatedAt
    );
    assert_eq!(
        ImprintField::from_str("UpdatedAt").unwrap(),
        ImprintField::UpdatedAt
    );
    assert!(ImprintField::from_str("ImprintID").is_err());
    assert!(ImprintField::from_str("Publisher").is_err());
    assert!(ImprintField::from_str("Website").is_err());
}
