use chrono::naive::NaiveDate;
use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
#[cfg(feature = "backend")]
use crate::schema::work;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Work_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkType {
    #[cfg_attr(feature = "backend", db_rename = "book-chapter")]
    BookChapter,
    Monograph,
    #[cfg_attr(feature = "backend", db_rename = "edited-book")]
    EditedBook,
    Textbook,
    #[cfg_attr(feature = "backend", db_rename = "journal-issue")]
    JournalIssue,
    #[cfg_attr(feature = "backend", db_rename = "book-set")]
    BookSet,
}

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Work_status")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkStatus {
    Unspecified,
    Cancelled,
    Forthcoming,
    #[cfg_attr(feature = "backend", db_rename = "postponed-indefinitely")]
    PostponedIndefinitely,
    Active,
    #[cfg_attr(feature = "backend", db_rename = "no-longer-our-product")]
    NoLongerOurProduct,
    #[cfg_attr(feature = "backend", db_rename = "out-of-stock-indefinitely")]
    OutOfStockIndefinitely,
    #[cfg_attr(feature = "backend", db_rename = "out-of-print")]
    OutOfPrint,
    Inactive,
    Unknown,
    Remaindered,
    #[cfg_attr(feature = "backend", db_rename = "withdrawn-from-sale")]
    WithdrawnFromSale,
    Recalled,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting works list")
)]
pub enum WorkField {
    WorkID,
    WorkType,
    WorkStatus,
    FullTitle,
    Title,
    Subtitle,
    Reference,
    Edition,
    ImprintID,
    DOI,
    PublicationDate,
    Place,
    Width,
    Height,
    PageCount,
    PageBreakdown,
    ImageCount,
    TableCount,
    AudioCount,
    VideoCount,
    License,
    CopyrightHolder,
    LandingPage,
    LCCN,
    OCLC,
    ShortAbstract,
    LongAbstract,
    GeneralNote,
    TOC,
    CoverURL,
    CoverCaption,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub imprint_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: String,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "work"
)]
pub struct NewWork {
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub imprint_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: String,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "work"
)]
pub struct PatchWork {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub imprint_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: String,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
}

impl Default for WorkType {
    fn default() -> WorkType {
        WorkType::Monograph
    }
}

impl Default for WorkStatus {
    fn default() -> WorkStatus {
        WorkStatus::Inactive
    }
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkType::BookChapter => write!(f, "Book Chapter"),
            WorkType::Monograph => write!(f, "Monograph"),
            WorkType::EditedBook => write!(f, "Edited Book"),
            WorkType::Textbook => write!(f, "Textbook"),
            WorkType::JournalIssue => write!(f, "Journal Issue"),
            WorkType::BookSet => write!(f, "Book Set"),
        }
    }
}

impl FromStr for WorkType {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<WorkType, ThothError> {
        match input {
            "Book Chapter" => Ok(WorkType::BookChapter),
            "Monograph" => Ok(WorkType::Monograph),
            "Edited Book" => Ok(WorkType::EditedBook),
            "Textbook" => Ok(WorkType::Textbook),
            "Journal Issue" => Ok(WorkType::JournalIssue),
            "Book Set" => Ok(WorkType::BookSet),
            _ => Err(ThothError::InvalidWorkType(input.to_string())),
        }
    }
}

impl fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkStatus::Unspecified => write!(f, "Unspecified"),
            WorkStatus::Cancelled => write!(f, "Cancelled"),
            WorkStatus::Forthcoming => write!(f, "Forthcoming"),
            WorkStatus::PostponedIndefinitely => write!(f, "Postponed Indefinitely"),
            WorkStatus::Active => write!(f, "Active"),
            WorkStatus::NoLongerOurProduct => write!(f, "No Longer Our Product"),
            WorkStatus::OutOfStockIndefinitely => write!(f, "Out Of Stock Indefinitely"),
            WorkStatus::OutOfPrint => write!(f, "Out Of Print"),
            WorkStatus::Inactive => write!(f, "Inactive"),
            WorkStatus::Unknown => write!(f, "Unknown"),
            WorkStatus::Remaindered => write!(f, "Remaindered"),
            WorkStatus::WithdrawnFromSale => write!(f, "Withdrawn From Sale"),
            WorkStatus::Recalled => write!(f, "Recalled"),
        }
    }
}

impl FromStr for WorkStatus {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<WorkStatus, ThothError> {
        match input {
            "Unspecified" => Ok(WorkStatus::Unspecified),
            "Cancelled" => Ok(WorkStatus::Cancelled),
            "Forthcoming" => Ok(WorkStatus::Forthcoming),
            "Postponed Indefinitely" => Ok(WorkStatus::PostponedIndefinitely),
            "Active" => Ok(WorkStatus::Active),
            "No Longer Our Product" => Ok(WorkStatus::NoLongerOurProduct),
            "Out Of Stock Indefinitely" => Ok(WorkStatus::OutOfStockIndefinitely),
            "Out Of Print" => Ok(WorkStatus::OutOfPrint),
            "Inactive" => Ok(WorkStatus::Inactive),
            "Unknown" => Ok(WorkStatus::Unknown),
            "Remaindered" => Ok(WorkStatus::Remaindered),
            "Withdrawn From Sale" => Ok(WorkStatus::WithdrawnFromSale),
            "Recalled" => Ok(WorkStatus::Recalled),
            _ => Err(ThothError::InvalidWorkStatus(input.to_string())),
        }
    }
}

#[test]
fn test_worktype_default() {
    let worktype: WorkType = Default::default();
    assert_eq!(worktype, WorkType::Monograph);
}

#[test]
fn test_workstatus_default() {
    let workstatus: WorkStatus = Default::default();
    assert_eq!(workstatus, WorkStatus::Inactive);
}

#[test]
fn test_worktype_display() {
    assert_eq!(format!("{}", WorkType::BookChapter), "Book Chapter");
    assert_eq!(format!("{}", WorkType::Monograph), "Monograph");
    assert_eq!(format!("{}", WorkType::EditedBook), "Edited Book");
    assert_eq!(format!("{}", WorkType::Textbook), "Textbook");
    assert_eq!(format!("{}", WorkType::JournalIssue), "Journal Issue");
    assert_eq!(format!("{}", WorkType::BookSet), "Book Set");
}

#[test]
fn test_workstatus_display() {
    assert_eq!(format!("{}", WorkStatus::Cancelled), "Cancelled");
    assert_eq!(format!("{}", WorkStatus::Forthcoming), "Forthcoming");
    assert_eq!(
        format!("{}", WorkStatus::PostponedIndefinitely),
        "Postponed Indefinitely"
    );
    assert_eq!(format!("{}", WorkStatus::Active), "Active");
    assert_eq!(
        format!("{}", WorkStatus::NoLongerOurProduct),
        "No Longer Our Product"
    );
    assert_eq!(
        format!("{}", WorkStatus::OutOfStockIndefinitely),
        "Out Of Stock Indefinitely"
    );
    assert_eq!(format!("{}", WorkStatus::OutOfPrint), "Out Of Print");
    assert_eq!(format!("{}", WorkStatus::Inactive), "Inactive");
    assert_eq!(format!("{}", WorkStatus::Unknown), "Unknown");
    assert_eq!(format!("{}", WorkStatus::Remaindered), "Remaindered");
    assert_eq!(
        format!("{}", WorkStatus::WithdrawnFromSale),
        "Withdrawn From Sale"
    );
    assert_eq!(format!("{}", WorkStatus::Recalled), "Recalled");
}

#[test]
fn test_worktype_fromstr() {
    assert_eq!(
        WorkType::from_str("Book Chapter").unwrap(),
        WorkType::BookChapter
    );
    assert_eq!(
        WorkType::from_str("Monograph").unwrap(),
        WorkType::Monograph
    );
    assert_eq!(
        WorkType::from_str("Edited Book").unwrap(),
        WorkType::EditedBook
    );
    assert_eq!(WorkType::from_str("Textbook").unwrap(), WorkType::Textbook);
    assert_eq!(
        WorkType::from_str("Journal Issue").unwrap(),
        WorkType::JournalIssue
    );
    assert_eq!(WorkType::from_str("Book Set").unwrap(), WorkType::BookSet);

    assert!(WorkType::from_str("Book Section").is_err());
    assert!(WorkType::from_str("Manuscript").is_err());
}

#[test]
fn test_workstatus_fromstr() {
    assert_eq!(
        WorkStatus::from_str("Unspecified").unwrap(),
        WorkStatus::Unspecified
    );
    assert_eq!(
        WorkStatus::from_str("Cancelled").unwrap(),
        WorkStatus::Cancelled
    );
    assert_eq!(
        WorkStatus::from_str("Forthcoming").unwrap(),
        WorkStatus::Forthcoming
    );
    assert_eq!(
        WorkStatus::from_str("Postponed Indefinitely").unwrap(),
        WorkStatus::PostponedIndefinitely
    );
    assert_eq!(WorkStatus::from_str("Active").unwrap(), WorkStatus::Active);
    assert_eq!(
        WorkStatus::from_str("No Longer Our Product").unwrap(),
        WorkStatus::NoLongerOurProduct
    );
    assert_eq!(
        WorkStatus::from_str("Out Of Stock Indefinitely").unwrap(),
        WorkStatus::OutOfStockIndefinitely
    );
    assert_eq!(
        WorkStatus::from_str("Out Of Print").unwrap(),
        WorkStatus::OutOfPrint
    );
    assert_eq!(
        WorkStatus::from_str("Inactive").unwrap(),
        WorkStatus::Inactive
    );
    assert_eq!(
        WorkStatus::from_str("Unknown").unwrap(),
        WorkStatus::Unknown
    );
    assert_eq!(
        WorkStatus::from_str("Remaindered").unwrap(),
        WorkStatus::Remaindered
    );
    assert_eq!(
        WorkStatus::from_str("Withdrawn From Sale").unwrap(),
        WorkStatus::WithdrawnFromSale
    );
    assert_eq!(
        WorkStatus::from_str("Recalled").unwrap(),
        WorkStatus::Recalled
    );

    assert!(WorkStatus::from_str("Published").is_err());
    assert!(WorkStatus::from_str("Unpublished").is_err());
}
