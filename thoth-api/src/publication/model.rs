use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
use crate::price::model::Price;
#[cfg(feature = "backend")]
use crate::schema::publication;
#[cfg(feature = "backend")]
use crate::schema::publication_history;
use crate::work::model::WorkWithRelations;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Publication_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationType {
    #[cfg_attr(feature = "backend", db_rename = "Paperback")]
    Paperback,
    #[cfg_attr(feature = "backend", db_rename = "Hardback")]
    Hardback,
    #[cfg_attr(feature = "backend", db_rename = "PDF")]
    #[strum(serialize = "PDF")]
    Pdf,
    #[cfg_attr(feature = "backend", db_rename = "HTML")]
    #[strum(serialize = "HTML")]
    Html,
    #[cfg_attr(feature = "backend", db_rename = "XML")]
    #[strum(serialize = "XML")]
    Xml,
    #[cfg_attr(feature = "backend", db_rename = "Epub")]
    Epub,
    #[cfg_attr(feature = "backend", db_rename = "Mobi")]
    Mobi,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting publications list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationField {
    #[strum(serialize = "ID")]
    PublicationId,
    #[strum(serialize = "Type")]
    PublicationType,
    #[strum(serialize = "WorkID")]
    WorkId,
    #[strum(serialize = "ISBN")]
    Isbn,
    #[strum(serialize = "URL")]
    PublicationUrl,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationWithRelations {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub updated_at: Timestamp,
    pub prices: Option<Vec<Price>>,
    pub work: WorkWithRelations,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "publication"
)]
pub struct NewPublication {
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "publication"
)]
pub struct PatchPublication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PublicationHistory {
    pub publication_history_id: Uuid,
    pub publication_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "publication_history"
)]
pub struct NewPublicationHistory {
    pub publication_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting publications list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PublicationOrderBy {
    pub field: PublicationField,
    pub direction: Direction,
}

impl Default for PublicationType {
    fn default() -> PublicationType {
        PublicationType::Paperback
    }
}

impl Default for PublicationField {
    fn default() -> Self {
        PublicationField::PublicationType
    }
}

#[test]
fn test_publicationtype_default() {
    let pubtype: PublicationType = Default::default();
    assert_eq!(pubtype, PublicationType::Paperback);
}

#[test]
fn test_publicationfield_default() {
    let pubfield: PublicationField = Default::default();
    assert_eq!(pubfield, PublicationField::PublicationType);
}

#[test]
fn test_publicationtype_display() {
    assert_eq!(format!("{}", PublicationType::Paperback), "Paperback");
    assert_eq!(format!("{}", PublicationType::Hardback), "Hardback");
    assert_eq!(format!("{}", PublicationType::Pdf), "PDF");
    assert_eq!(format!("{}", PublicationType::Html), "HTML");
    assert_eq!(format!("{}", PublicationType::Xml), "XML");
    assert_eq!(format!("{}", PublicationType::Epub), "Epub");
    assert_eq!(format!("{}", PublicationType::Mobi), "Mobi");
}

#[test]
fn test_publicationfield_display() {
    assert_eq!(format!("{}", PublicationField::PublicationId), "ID");
    assert_eq!(format!("{}", PublicationField::PublicationType), "Type");
    assert_eq!(format!("{}", PublicationField::WorkId), "WorkID");
    assert_eq!(format!("{}", PublicationField::Isbn), "ISBN");
    assert_eq!(format!("{}", PublicationField::PublicationUrl), "URL");
    assert_eq!(format!("{}", PublicationField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", PublicationField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_publicationtype_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        PublicationType::from_str("Paperback").unwrap(),
        PublicationType::Paperback
    );
    assert_eq!(
        PublicationType::from_str("Hardback").unwrap(),
        PublicationType::Hardback
    );
    assert_eq!(
        PublicationType::from_str("PDF").unwrap(),
        PublicationType::Pdf
    );
    assert_eq!(
        PublicationType::from_str("HTML").unwrap(),
        PublicationType::Html
    );
    assert_eq!(
        PublicationType::from_str("XML").unwrap(),
        PublicationType::Xml
    );
    assert_eq!(
        PublicationType::from_str("Epub").unwrap(),
        PublicationType::Epub
    );
    assert_eq!(
        PublicationType::from_str("Mobi").unwrap(),
        PublicationType::Mobi
    );

    assert!(PublicationType::from_str("PNG").is_err());
    assert!(PublicationType::from_str("Latex").is_err());
}

#[test]
fn test_publicationfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        PublicationField::from_str("ID").unwrap(),
        PublicationField::PublicationId
    );
    assert_eq!(
        PublicationField::from_str("Type").unwrap(),
        PublicationField::PublicationType
    );
    assert_eq!(
        PublicationField::from_str("WorkID").unwrap(),
        PublicationField::WorkId
    );
    assert_eq!(
        PublicationField::from_str("ISBN").unwrap(),
        PublicationField::Isbn
    );
    assert_eq!(
        PublicationField::from_str("URL").unwrap(),
        PublicationField::PublicationUrl
    );
    assert_eq!(
        PublicationField::from_str("CreatedAt").unwrap(),
        PublicationField::CreatedAt
    );
    assert_eq!(
        PublicationField::from_str("UpdatedAt").unwrap(),
        PublicationField::UpdatedAt
    );
    assert!(PublicationField::from_str("PublicationID").is_err());
    assert!(PublicationField::from_str("Work Title").is_err());
    assert!(PublicationField::from_str("Work DOI").is_err());
}
