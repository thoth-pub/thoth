use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
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
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Standardised specification for accessibility to which a publication may conform"
    ),
    ExistingTypePath = "crate::schema::sql_types::AccessibilityStandard"
)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessibilityStandard {
    #[cfg_attr(
        feature = "backend",
        db_rename = "wcag-21-aa",
        graphql(description = "WCAG 2.1 AA")
    )]
    Wcag21aa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "wcag-21-aaa",
        graphql(description = "WCAG 2.1 AAA")
    )]
    Wcag21aaa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "wcag-22-aa",
        graphql(description = "WCAG 2.2 AA")
    )]
    Wcag22aa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "wcag-22-aaa",
        graphql(description = "WCAG 2.2 AAA")
    )]
    Wcag22aaa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "epub-a11y-10-aa",
        graphql(description = "EPUB Accessibility Specification 1.0 AA")
    )]
    EpubA11y10aa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "epub-a11y-10-aaa",
        graphql(description = "EPUB Accessibility Specification 1.0 AAA")
    )]
    EpubA11y10aaa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "epub-a11y-11-aa",
        graphql(description = "EPUB Accessibility Specification 1.1 AA")
    )]
    EpubA11y11aa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "epub-a11y-11-aaa",
        graphql(description = "EPUB Accessibility Specification 1.1 AAA")
    )]
    EpubA11y11aaa,
    #[cfg_attr(
        feature = "backend",
        db_rename = "pdf-ua-1",
        graphql(description = "PDF/UA-1")
    )]
    PdfUa1,
    #[cfg_attr(
        feature = "backend",
        db_rename = "pdf-ua-2",
        graphql(description = "PDF/UA-2")
    )]
    PdfUa2,
}

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Reason for publication not being required to comply with accessibility standards"
    ),
    ExistingTypePath = "crate::schema::sql_types::AccessibilityException"
)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessibilityException {
    #[cfg_attr(
        feature = "backend",
        db_rename = "micro-enterprises",
        graphql(description = "Publisher is a micro-enterprise")
    )]
    MicroEnterprises,
    #[cfg_attr(
        feature = "backend",
        db_rename = "disproportionate-burden",
        graphql(
            description = "Making the publication accessible would financially overburden the publisher"
        )
    )]
    DisproportionateBurden,
    #[cfg_attr(
        feature = "backend",
        db_rename = "fundamental-alteration",
        graphql(
            description = "Making the publication accessible would fundamentally modify the nature of it"
        )
    )]
    FundamentalAlteration,
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
    AccessibilityStandard,
    AccessibilityAdditionalStandard,
    AccessibilityException,
    AccessibilityReportUrl,
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
    pub accessibility_standard: Option<AccessibilityStandard>,
    pub accessibility_additional_standard: Option<AccessibilityStandard>,
    pub accessibility_exception: Option<AccessibilityException>,
    pub accessibility_report_url: Option<String>,
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
    pub accessibility_standard: Option<AccessibilityStandard>,
    pub accessibility_additional_standard: Option<AccessibilityStandard>,
    pub accessibility_exception: Option<AccessibilityException>,
    pub accessibility_report_url: Option<String>,
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
    pub accessibility_standard: Option<AccessibilityStandard>,
    pub accessibility_additional_standard: Option<AccessibilityStandard>,
    pub accessibility_exception: Option<AccessibilityException>,
    pub accessibility_report_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PublicationHistory {
    pub publication_history_id: Uuid,
    pub publication_id: Uuid,
    pub user_id: String,
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
    pub user_id: String,
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
        matches!(
            self.publication_type(),
            PublicationType::Paperback | PublicationType::Hardback
        )
    }

    fn is_digital(&self) -> bool {
        !self.is_physical()
    }

    fn has_dimension(&self) -> bool {
        [
            self.width_mm(),
            self.width_in(),
            self.height_mm(),
            self.height_in(),
            self.depth_mm(),
            self.depth_in(),
            self.weight_g(),
            self.weight_oz(),
        ]
        .into_iter()
        .any(Option::is_some)
    }

    fn validate_dimensions_constraints(&self) -> ThothResult<()> {
        use ThothError::*;

        if self.is_digital() && self.has_dimension() {
            return Err(DimensionDigitalError);
        }

        // If value in one unit is supplied, the other cannot be left empty.
        for (metric, imperial, err) in [
            (self.width_mm(), self.width_in(), WidthEmptyError),
            (self.height_mm(), self.height_in(), HeightEmptyError),
            (self.depth_mm(), self.depth_in(), DepthEmptyError),
            (self.weight_g(), self.weight_oz(), WeightEmptyError),
        ] {
            if metric.is_some() ^ imperial.is_some() {
                return Err(err);
            }
        }
        Ok(())
    }

    #[cfg(feature = "backend")]
    fn is_chapter(&self, db: &crate::db::PgPool) -> ThothResult<bool> {
        use crate::model::work::WorkType;
        use diesel::prelude::*;
        let mut connection = db.get()?;
        let work_type = crate::schema::work::table
            .select(crate::schema::work::work_type)
            .filter(crate::schema::work::work_id.eq(self.work_id()))
            .first::<WorkType>(&mut connection)?;
        Ok(work_type == WorkType::BookChapter)
    }

    #[cfg(feature = "backend")]
    fn validate_chapter_constraints(&self) -> ThothResult<()> {
        match (self.isbn().is_some(), self.has_dimension()) {
            (true, _) => Err(ThothError::ChapterIsbnError),
            (_, true) => Err(ThothError::ChapterDimensionError),
            _ => Ok(()),
        }
    }

    #[cfg(feature = "backend")]
    fn validate(&self, db: &crate::db::PgPool) -> ThothResult<()> {
        if self.is_chapter(db)? {
            self.validate_chapter_constraints()?;
        }
        self.validate_dimensions_constraints()
    }
}

macro_rules! publication_properties {
    ($t:ty) => {
        impl PublicationProperties for $t {
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
    };
}
publication_properties!(Publication);
publication_properties!(NewPublication);
publication_properties!(PatchPublication);

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::PublicationPolicy;
#[cfg(test)]
mod tests;
