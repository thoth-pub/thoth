use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::location::Location;
use crate::model::price::Price;
use crate::model::work::WorkWithRelations;
use crate::model::Isbn;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::publication;
#[cfg(feature = "backend")]
use crate::schema::publication_history;

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
    #[cfg_attr(feature = "backend", db_rename = "AZW3")]
    #[strum(serialize = "AZW3")]
    Azw3,
    #[cfg_attr(feature = "backend", db_rename = "DOCX")]
    #[strum(serialize = "DOCX")]
    Docx,
    #[cfg_attr(feature = "backend", db_rename = "FictionBook")]
    FictionBook,
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
    CreatedAt,
    UpdatedAt,
    WeightG,
    WeightOz,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationWithRelations {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub updated_at: Timestamp,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
    pub prices: Option<Vec<Price>>,
    pub locations: Option<Vec<Location>>,
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
    pub isbn: Option<Isbn>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
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
    pub isbn: Option<Isbn>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
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

impl PublicationType {
    fn is_physical(&self) -> bool {
        matches!(self, PublicationType::Paperback | PublicationType::Hardback)
    }

    fn is_digital(&self) -> bool {
        !self.is_physical()
    }
}

pub trait PublicationProperties {
    fn publication_type(&self) -> &PublicationType;
    fn weight_g(&self) -> &Option<f64>;
    fn weight_oz(&self) -> &Option<f64>;
    fn isbn(&self) -> &Option<Isbn>;
    fn work_id(&self) -> &Uuid;

    fn is_physical(&self) -> bool {
        self.publication_type().is_physical()
    }

    fn is_digital(&self) -> bool {
        self.publication_type().is_digital()
    }

    fn weight_error(&self) -> ThothResult<()> {
        if self.is_digital() {
            // Digital publications cannot have weight values.
            if self.weight_g().is_some() || self.weight_oz().is_some() {
                return Err(ThothError::WeightDigitalError);
            }
        } else if (self.weight_g().is_some() && self.weight_oz().is_none())
            || (self.weight_oz().is_some() && self.weight_g().is_none())
        {
            // If one weight value is supplied, the other cannot be left empty.
            return Err(ThothError::WeightEmptyError);
        }
        Ok(())
    }
}

impl PublicationProperties for Publication {
    fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    fn weight_g(&self) -> &Option<f64> {
        &self.weight_g
    }

    fn weight_oz(&self) -> &Option<f64> {
        &self.weight_oz
    }

    fn isbn(&self) -> &Option<Isbn> {
        &self.isbn
    }

    fn work_id(&self) -> &Uuid {
        &self.work_id
    }
}

impl PublicationProperties for PublicationWithRelations {
    fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    fn weight_g(&self) -> &Option<f64> {
        &self.weight_g
    }

    fn weight_oz(&self) -> &Option<f64> {
        &self.weight_oz
    }

    fn isbn(&self) -> &Option<Isbn> {
        &self.isbn
    }

    fn work_id(&self) -> &Uuid {
        &self.work_id
    }
}

impl PublicationProperties for NewPublication {
    fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    fn weight_g(&self) -> &Option<f64> {
        &self.weight_g
    }

    fn weight_oz(&self) -> &Option<f64> {
        &self.weight_oz
    }

    fn isbn(&self) -> &Option<Isbn> {
        &self.isbn
    }

    fn work_id(&self) -> &Uuid {
        &self.work_id
    }
}

impl PublicationProperties for PatchPublication {
    fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    fn weight_g(&self) -> &Option<f64> {
        &self.weight_g
    }

    fn weight_oz(&self) -> &Option<f64> {
        &self.weight_oz
    }

    fn isbn(&self) -> &Option<Isbn> {
        &self.isbn
    }

    fn work_id(&self) -> &Uuid {
        &self.work_id
    }
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
fn test_publicationproperties_type() {
    let mut publication: Publication = Default::default();
    for pub_type in [PublicationType::Paperback, PublicationType::Hardback] {
        publication.publication_type = pub_type;
        assert!(publication.publication_type.is_physical());
        assert!(!publication.publication_type.is_digital());
        assert!(publication.is_physical());
        assert!(!publication.is_digital());
    }
    for pub_type in [
        PublicationType::Azw3,
        PublicationType::Docx,
        PublicationType::Epub,
        PublicationType::FictionBook,
        PublicationType::Html,
        PublicationType::Mobi,
        PublicationType::Pdf,
        PublicationType::Xml,
    ] {
        publication.publication_type = pub_type;
        assert!(!publication.publication_type.is_physical());
        assert!(publication.publication_type.is_digital());
        assert!(!publication.is_physical());
        assert!(publication.is_digital());
    }
}

#[test]
fn test_publicationproperties_weight() {
    let mut publication: Publication = Publication {
        publication_type: PublicationType::Pdf,
        weight_g: Some(100.0),
        ..Default::default()
    };
    assert_eq!(
        publication.weight_error(),
        Err(ThothError::WeightDigitalError)
    );
    publication.weight_g = None;
    assert_eq!(publication.weight_error(), Ok(()));
    publication.weight_oz = Some(3.5);
    assert_eq!(
        publication.weight_error(),
        Err(ThothError::WeightDigitalError)
    );
    publication.publication_type = PublicationType::Paperback;
    assert_eq!(
        publication.weight_error(),
        Err(ThothError::WeightEmptyError)
    );
    publication.weight_oz = None;
    assert_eq!(publication.weight_error(), Ok(()));
    publication.weight_g = Some(100.0);
    assert_eq!(
        publication.weight_error(),
        Err(ThothError::WeightEmptyError)
    );
    publication.weight_oz = Some(3.5);
    assert_eq!(publication.weight_error(), Ok(()));
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
    assert_eq!(format!("{}", PublicationType::Azw3), "AZW3");
    assert_eq!(format!("{}", PublicationType::Docx), "DOCX");
    assert_eq!(format!("{}", PublicationType::FictionBook), "FictionBook");
}

#[test]
fn test_publicationfield_display() {
    assert_eq!(format!("{}", PublicationField::PublicationId), "ID");
    assert_eq!(format!("{}", PublicationField::PublicationType), "Type");
    assert_eq!(format!("{}", PublicationField::WorkId), "WorkID");
    assert_eq!(format!("{}", PublicationField::Isbn), "ISBN");
    assert_eq!(format!("{}", PublicationField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", PublicationField::UpdatedAt), "UpdatedAt");
    assert_eq!(format!("{}", PublicationField::WeightG), "WeightG");
    assert_eq!(format!("{}", PublicationField::WeightOz), "WeightOz");
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
    assert_eq!(
        PublicationType::from_str("AZW3").unwrap(),
        PublicationType::Azw3
    );
    assert_eq!(
        PublicationType::from_str("DOCX").unwrap(),
        PublicationType::Docx
    );
    assert_eq!(
        PublicationType::from_str("FictionBook").unwrap(),
        PublicationType::FictionBook
    );

    assert!(PublicationType::from_str("PNG").is_err());
    assert!(PublicationType::from_str("Latex").is_err());
    assert!(PublicationType::from_str("azw3").is_err());
    assert!(PublicationType::from_str("Fiction Book").is_err());
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
        PublicationField::from_str("CreatedAt").unwrap(),
        PublicationField::CreatedAt
    );
    assert_eq!(
        PublicationField::from_str("UpdatedAt").unwrap(),
        PublicationField::UpdatedAt
    );
    assert_eq!(
        PublicationField::from_str("WeightG").unwrap(),
        PublicationField::WeightG
    );
    assert_eq!(
        PublicationField::from_str("WeightOz").unwrap(),
        PublicationField::WeightOz
    );
    assert!(PublicationField::from_str("PublicationID").is_err());
    assert!(PublicationField::from_str("Work Title").is_err());
    assert!(PublicationField::from_str("Work DOI").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
