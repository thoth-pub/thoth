use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
#[cfg(feature = "backend")]
use crate::schema::publication;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Publication_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationType {
    #[cfg_attr(feature = "backend", db_rename = "Paperback")]
    Paperback,
    #[cfg_attr(feature = "backend", db_rename = "Hardback")]
    Hardback,
    #[cfg_attr(feature = "backend", db_rename = "PDF")]
    #[serde(rename = "PDF")]
    PDF,
    #[cfg_attr(feature = "backend", db_rename = "HTML")]
    #[serde(rename = "HTML")]
    HTML,
    #[cfg_attr(feature = "backend", db_rename = "XML")]
    #[serde(rename = "XML")]
    XML,
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
pub enum PublicationField {
    PublicationID,
    PublicationType,
    WorkID,
    ISBN,
    PublicationURL,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

impl Default for PublicationType {
    fn default() -> PublicationType {
        PublicationType::Paperback
    }
}

impl fmt::Display for PublicationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PublicationType::Paperback => write!(f, "Paperback"),
            PublicationType::Hardback => write!(f, "Hardback"),
            PublicationType::PDF => write!(f, "PDF"),
            PublicationType::HTML => write!(f, "HTML"),
            PublicationType::XML => write!(f, "XML"),
            PublicationType::Epub => write!(f, "Epub"),
            PublicationType::Mobi => write!(f, "Mobi"),
        }
    }
}

impl FromStr for PublicationType {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<PublicationType, ThothError> {
        match input {
            "Paperback" => Ok(PublicationType::Paperback),
            "Hardback" => Ok(PublicationType::Hardback),
            "PDF" => Ok(PublicationType::PDF),
            "HTML" => Ok(PublicationType::HTML),
            "XML" => Ok(PublicationType::XML),
            "Epub" => Ok(PublicationType::Epub),
            "Mobi" => Ok(PublicationType::Mobi),
            _ => Err(ThothError::InvalidPublicationType(input.to_string())),
        }
    }
}

#[test]
fn test_publicationtype_default() {
    let pubtype: PublicationType = Default::default();
    assert_eq!(pubtype, PublicationType::Paperback);
}

#[test]
fn test_publicationtype_display() {
    assert_eq!(format!("{}", PublicationType::Paperback), "Paperback");
    assert_eq!(format!("{}", PublicationType::Hardback), "Hardback");
    assert_eq!(format!("{}", PublicationType::PDF), "PDF");
    assert_eq!(format!("{}", PublicationType::HTML), "HTML");
    assert_eq!(format!("{}", PublicationType::XML), "XML");
    assert_eq!(format!("{}", PublicationType::Epub), "Epub");
    assert_eq!(format!("{}", PublicationType::Mobi), "Mobi");
}

#[test]
fn test_publicationtype_fromstr() {
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
        PublicationType::PDF
    );
    assert_eq!(
        PublicationType::from_str("HTML").unwrap(),
        PublicationType::HTML
    );
    assert_eq!(
        PublicationType::from_str("XML").unwrap(),
        PublicationType::XML
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
