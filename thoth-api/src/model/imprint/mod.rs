use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::publisher::Publisher;
use crate::model::Timestamp;
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
    #[strum(serialize = "ID")]
    ImprintId,
    #[strum(serialize = "Imprint")]
    ImprintName,
    #[strum(serialize = "ImprintURL")]
    ImprintUrl,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Imprint {
    pub imprint_id: Uuid,
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImprintWithPublisher {
    pub imprint_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub updated_at: Timestamp,
    pub publisher: Publisher,
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
    pub timestamp: Timestamp,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
    assert_eq!(format!("{}", ImprintField::ImprintId), "ID");
    assert_eq!(format!("{}", ImprintField::ImprintName), "Imprint");
    assert_eq!(format!("{}", ImprintField::ImprintUrl), "ImprintURL");
    assert_eq!(format!("{}", ImprintField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", ImprintField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_imprintfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        ImprintField::from_str("ID").unwrap(),
        ImprintField::ImprintId
    );
    assert_eq!(
        ImprintField::from_str("Imprint").unwrap(),
        ImprintField::ImprintName
    );
    assert_eq!(
        ImprintField::from_str("ImprintURL").unwrap(),
        ImprintField::ImprintUrl
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

#[cfg(feature = "backend")]
pub mod crud;
