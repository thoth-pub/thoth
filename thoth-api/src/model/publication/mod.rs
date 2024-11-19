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

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Format of a publication"),
    ExistingTypePath = "crate::schema::sql_types::PublicationType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationType {
    #[cfg_attr(
        feature = "backend",
        db_rename = "Paperback",
        graphql(description = "Paperback print format")
    )]
    #[default]
    Paperback,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Hardback",
        graphql(description = "Hardback print format")
    )]
    Hardback,
    #[cfg_attr(
        feature = "backend",
        db_rename = "PDF",
        graphql(description = "PDF ebook format")
    )]
    #[strum(serialize = "PDF")]
    Pdf,
    #[cfg_attr(
        feature = "backend",
        db_rename = "HTML",
        graphql(description = "HTML ebook format")
    )]
    #[strum(serialize = "HTML")]
    Html,
    #[cfg_attr(
        feature = "backend",
        db_rename = "XML",
        graphql(description = "XML ebook format")
    )]
    #[strum(serialize = "XML")]
    Xml,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Epub",
        graphql(description = "Epub ebook format")
    )]
    Epub,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Mobi",
        graphql(description = "Mobipocket (.mobi) ebook format")
    )]
    Mobi,
    #[cfg_attr(
        feature = "backend",
        db_rename = "AZW3",
        graphql(description = "Kindle version 8 (.azw3) ebook format")
    )]
    #[strum(serialize = "AZW3")]
    Azw3,
    #[cfg_attr(
        feature = "backend",
        db_rename = "DOCX",
        graphql(description = "Microsoft Word (.docx) ebook format")
    )]
    #[strum(serialize = "DOCX")]
    Docx,
    #[cfg_attr(
        feature = "backend",
        db_rename = "FictionBook",
        graphql(description = "FictionBook (.fb2, .fb3, .fbz) ebook format")
    )]
    FictionBook,
    #[cfg_attr(
        feature = "backend",
        db_rename = "MP3",
        graphql(description = "MP3 audiobook format")
    )]
    #[strum(serialize = "MP3")]
    Mp3,
    #[cfg_attr(
        feature = "backend",
        db_rename = "WAV",
        graphql(description = "WAV audiobook format")
    )]
    #[strum(serialize = "WAV")]
    Wav,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting publications list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationField {
    #[strum(serialize = "ID")]
    PublicationId,
    #[strum(serialize = "Type")]
    #[default]
    PublicationType,
    #[strum(serialize = "WorkID")]
    WorkId,
    #[strum(serialize = "ISBN")]
    Isbn,
    CreatedAt,
    UpdatedAt,
    WidthMm,
    WidthIn,
    HeightMm,
    HeightIn,
    DepthMm,
    DepthIn,
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
    pub width_mm: Option<f64>,
    pub width_in: Option<f64>,
    pub height_mm: Option<f64>,
    pub height_in: Option<f64>,
    pub depth_mm: Option<f64>,
    pub depth_in: Option<f64>,
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
    pub width_mm: Option<f64>,
    pub width_in: Option<f64>,
    pub height_mm: Option<f64>,
    pub height_in: Option<f64>,
    pub depth_mm: Option<f64>,
    pub depth_in: Option<f64>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
    pub prices: Option<Vec<Price>>,
    pub locations: Option<Vec<Location>>,
    pub work: WorkWithRelations,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new manifestation of a written text"),
    diesel(table_name = publication)
)]
pub struct NewPublication {
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub width_mm: Option<f64>,
    pub width_in: Option<f64>,
    pub height_mm: Option<f64>,
    pub height_in: Option<f64>,
    pub depth_mm: Option<f64>,
    pub depth_in: Option<f64>,
    pub weight_g: Option<f64>,
    pub weight_oz: Option<f64>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing manifestation of a written text"),
    diesel(table_name = publication, treat_none_as_null = true)
)]
pub struct PatchPublication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<Isbn>,
    pub width_mm: Option<f64>,
    pub width_in: Option<f64>,
    pub height_mm: Option<f64>,
    pub height_in: Option<f64>,
    pub depth_mm: Option<f64>,
    pub depth_in: Option<f64>,
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
    diesel(table_name = publication_history)
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    fn width_mm(&self) -> &Option<f64>;
    fn width_in(&self) -> &Option<f64>;
    fn height_mm(&self) -> &Option<f64>;
    fn height_in(&self) -> &Option<f64>;
    fn depth_mm(&self) -> &Option<f64>;
    fn depth_in(&self) -> &Option<f64>;
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

    fn has_dimension(&self) -> bool {
        self.width_mm().is_some()
            || self.width_in().is_some()
            || self.height_mm().is_some()
            || self.height_in().is_some()
            || self.depth_mm().is_some()
            || self.depth_in().is_some()
            || self.weight_g().is_some()
            || self.weight_oz().is_some()
    }

    fn dimension_error(&self) -> ThothResult<()> {
        if self.is_digital() {
            // Digital publications cannot have dimension values.
            if self.has_dimension() {
                return Err(ThothError::DimensionDigitalError);
            }
        } else if (self.width_mm().is_some() && self.width_in().is_none())
            || (self.width_in().is_some() && self.width_mm().is_none())
        {
            // If one width value is supplied, the other cannot be left empty.
            return Err(ThothError::WidthEmptyError);
        } else if (self.height_mm().is_some() && self.height_in().is_none())
            || (self.height_in().is_some() && self.height_mm().is_none())
        {
            // If one height value is supplied, the other cannot be left empty.
            return Err(ThothError::HeightEmptyError);
        } else if (self.depth_mm().is_some() && self.depth_in().is_none())
            || (self.depth_in().is_some() && self.depth_mm().is_none())
        {
            // If one depth value is supplied, the other cannot be left empty.
            return Err(ThothError::DepthEmptyError);
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

    fn width_mm(&self) -> &Option<f64> {
        &self.width_mm
    }

    fn width_in(&self) -> &Option<f64> {
        &self.width_in
    }

    fn height_mm(&self) -> &Option<f64> {
        &self.height_mm
    }

    fn height_in(&self) -> &Option<f64> {
        &self.height_in
    }

    fn depth_mm(&self) -> &Option<f64> {
        &self.depth_mm
    }

    fn depth_in(&self) -> &Option<f64> {
        &self.depth_in
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

    fn width_mm(&self) -> &Option<f64> {
        &self.width_mm
    }

    fn width_in(&self) -> &Option<f64> {
        &self.width_in
    }

    fn height_mm(&self) -> &Option<f64> {
        &self.height_mm
    }

    fn height_in(&self) -> &Option<f64> {
        &self.height_in
    }

    fn depth_mm(&self) -> &Option<f64> {
        &self.depth_mm
    }

    fn depth_in(&self) -> &Option<f64> {
        &self.depth_in
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

    fn width_mm(&self) -> &Option<f64> {
        &self.width_mm
    }

    fn width_in(&self) -> &Option<f64> {
        &self.width_in
    }

    fn height_mm(&self) -> &Option<f64> {
        &self.height_mm
    }

    fn height_in(&self) -> &Option<f64> {
        &self.height_in
    }

    fn depth_mm(&self) -> &Option<f64> {
        &self.depth_mm
    }

    fn depth_in(&self) -> &Option<f64> {
        &self.depth_in
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

    fn width_mm(&self) -> &Option<f64> {
        &self.width_mm
    }

    fn width_in(&self) -> &Option<f64> {
        &self.width_in
    }

    fn height_mm(&self) -> &Option<f64> {
        &self.height_mm
    }

    fn height_in(&self) -> &Option<f64> {
        &self.height_in
    }

    fn depth_mm(&self) -> &Option<f64> {
        &self.depth_mm
    }

    fn depth_in(&self) -> &Option<f64> {
        &self.depth_in
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
        PublicationType::Mp3,
        PublicationType::Pdf,
        PublicationType::Xml,
        PublicationType::Wav,
    ] {
        publication.publication_type = pub_type;
        assert!(!publication.publication_type.is_physical());
        assert!(publication.publication_type.is_digital());
        assert!(!publication.is_physical());
        assert!(publication.is_digital());
    }
}

#[test]
fn test_publicationproperties_width() {
    let mut publication: Publication = Publication {
        publication_type: PublicationType::Pdf,
        width_mm: Some(100.0),
        ..Default::default()
    };
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.width_mm = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.width_in = Some(39.4);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.publication_type = PublicationType::Paperback;
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::WidthEmptyError)
    );
    publication.width_in = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.width_mm = Some(100.0);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::WidthEmptyError)
    );
    publication.width_in = Some(39.4);
    assert_eq!(publication.dimension_error(), Ok(()));
}

#[test]
fn test_publicationproperties_height() {
    let mut publication: Publication = Publication {
        publication_type: PublicationType::Pdf,
        height_mm: Some(100.0),
        ..Default::default()
    };
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.height_mm = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.height_in = Some(39.4);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.publication_type = PublicationType::Paperback;
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::HeightEmptyError)
    );
    publication.height_in = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.height_mm = Some(100.0);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::HeightEmptyError)
    );
    publication.height_in = Some(39.4);
    assert_eq!(publication.dimension_error(), Ok(()));
}

#[test]
fn test_publicationproperties_depth() {
    let mut publication: Publication = Publication {
        publication_type: PublicationType::Pdf,
        depth_mm: Some(10.0),
        ..Default::default()
    };
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.depth_mm = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.depth_in = Some(3.94);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.publication_type = PublicationType::Paperback;
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DepthEmptyError)
    );
    publication.depth_in = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.depth_mm = Some(10.0);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DepthEmptyError)
    );
    publication.depth_in = Some(3.94);
    assert_eq!(publication.dimension_error(), Ok(()));
}

#[test]
fn test_publicationproperties_weight() {
    let mut publication: Publication = Publication {
        publication_type: PublicationType::Pdf,
        weight_g: Some(100.0),
        ..Default::default()
    };
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.weight_g = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.weight_oz = Some(3.5);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::DimensionDigitalError)
    );
    publication.publication_type = PublicationType::Paperback;
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::WeightEmptyError)
    );
    publication.weight_oz = None;
    assert_eq!(publication.dimension_error(), Ok(()));
    publication.weight_g = Some(100.0);
    assert_eq!(
        publication.dimension_error(),
        Err(ThothError::WeightEmptyError)
    );
    publication.weight_oz = Some(3.5);
    assert_eq!(publication.dimension_error(), Ok(()));
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
    assert_eq!(format!("{}", PublicationType::Mp3), "MP3");
    assert_eq!(format!("{}", PublicationType::Wav), "WAV");
}

#[test]
fn test_publicationfield_display() {
    assert_eq!(format!("{}", PublicationField::PublicationId), "ID");
    assert_eq!(format!("{}", PublicationField::PublicationType), "Type");
    assert_eq!(format!("{}", PublicationField::WorkId), "WorkID");
    assert_eq!(format!("{}", PublicationField::Isbn), "ISBN");
    assert_eq!(format!("{}", PublicationField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", PublicationField::UpdatedAt), "UpdatedAt");
    assert_eq!(format!("{}", PublicationField::WidthMm), "WidthMm");
    assert_eq!(format!("{}", PublicationField::WidthIn), "WidthIn");
    assert_eq!(format!("{}", PublicationField::HeightMm), "HeightMm");
    assert_eq!(format!("{}", PublicationField::HeightIn), "HeightIn");
    assert_eq!(format!("{}", PublicationField::DepthMm), "DepthMm");
    assert_eq!(format!("{}", PublicationField::DepthIn), "DepthIn");
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

    assert_eq!(
        PublicationType::from_str("MP3").unwrap(),
        PublicationType::Mp3
    );
    assert_eq!(
        PublicationType::from_str("WAV").unwrap(),
        PublicationType::Wav
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
        PublicationField::from_str("WidthMm").unwrap(),
        PublicationField::WidthMm
    );
    assert_eq!(
        PublicationField::from_str("WidthIn").unwrap(),
        PublicationField::WidthIn
    );
    assert_eq!(
        PublicationField::from_str("HeightMm").unwrap(),
        PublicationField::HeightMm
    );
    assert_eq!(
        PublicationField::from_str("HeightIn").unwrap(),
        PublicationField::HeightIn
    );
    assert_eq!(
        PublicationField::from_str("DepthMm").unwrap(),
        PublicationField::DepthMm
    );
    assert_eq!(
        PublicationField::from_str("DepthIn").unwrap(),
        PublicationField::DepthIn
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
