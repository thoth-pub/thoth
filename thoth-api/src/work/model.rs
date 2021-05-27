use chrono::naive::NaiveDate;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::contribution::model::Contribution;
use crate::errors::{ThothError, ThothResult};
use crate::funding::model::FundingExtended as Funding;
use crate::graphql::utils::Direction;
use crate::imprint::model::ImprintExtended as Imprint;
use crate::issue::model::IssueExtended as Issue;
use crate::language::model::Language;
use crate::publication::model::PublicationExtended as Publication;
#[cfg(feature = "backend")]
use crate::schema::work;
#[cfg(feature = "backend")]
use crate::schema::work_history;
use crate::subject::model::Subject;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Work_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkField {
    #[strum(serialize = "ID")]
    WorkId,
    #[strum(serialize = "Type")]
    WorkType,
    WorkStatus,
    #[strum(serialize = "Title")]
    FullTitle,
    #[strum(serialize = "ShortTitle")]
    Title,
    Subtitle,
    Reference,
    Edition,
    #[strum(serialize = "DOI")]
    Doi,
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
    #[strum(serialize = "LCCN")]
    Lccn,
    #[strum(serialize = "OCLC")]
    Oclc,
    ShortAbstract,
    LongAbstract,
    GeneralNote,
    #[strum(serialize = "TOC")]
    Toc,
    #[strum(serialize = "CoverURL")]
    CoverUrl,
    CoverCaption,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(
    feature = "backend",
    derive(DieselNewType, juniper::GraphQLScalarValue)
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Doi(String);

pub const DOI_DOMAIN: &str = "https://doi.org/";

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
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
    pub doi: Option<Doi>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkExtended {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub doi: Option<Doi>,
    pub publication_date: Option<String>,
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
    pub updated_at: DateTime<Utc>,
    pub contributions: Option<Vec<Contribution>>,
    pub publications: Option<Vec<Publication>>,
    pub languages: Option<Vec<Language>>,
    pub fundings: Option<Vec<Funding>>,
    pub subjects: Option<Vec<Subject>>,
    pub issues: Option<Vec<Issue>>,
    pub imprint: Imprint,
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
    pub doi: Option<Doi>,
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
    pub doi: Option<Doi>,
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

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct WorkHistory {
    pub work_history_id: Uuid,
    pub work_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[cfg_attr(feature = "backend", derive(Insertable), table_name = "work_history")]
pub struct NewWorkHistory {
    pub work_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting works list")
)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkOrderBy {
    pub field: WorkField,
    pub direction: Direction,
}

impl WorkExtended {
    pub fn compile_fulltitle(&self) -> String {
        if let Some(subtitle) = &self.subtitle.clone() {
            format!("{}: {}", self.title, subtitle)
        } else {
            self.title.to_string()
        }
    }

    pub fn publisher(&self) -> String {
        if let Some(short_name) = &self.imprint.publisher.publisher_shortname.clone() {
            short_name.to_string()
        } else {
            self.imprint.publisher.publisher_name.to_string()
        }
    }
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

impl Default for WorkField {
    fn default() -> Self {
        WorkField::FullTitle
    }
}

impl Default for WorkExtended {
    fn default() -> WorkExtended {
        WorkExtended {
            work_id: Default::default(),
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Inactive,
            full_title: "".to_string(),
            title: "".to_string(),
            subtitle: None,
            reference: None,
            edition: 1,
            doi: None,
            publication_date: None,
            place: None,
            width: None,
            height: None,
            page_count: None,
            page_breakdown: None,
            image_count: None,
            table_count: None,
            audio_count: None,
            video_count: None,
            license: None,
            copyright_holder: "".to_string(),
            landing_page: None,
            lccn: None,
            oclc: None,
            short_abstract: None,
            long_abstract: None,
            general_note: None,
            toc: None,
            cover_url: None,
            cover_caption: None,
            updated_at: chrono::TimeZone::timestamp(&Utc, 0, 0),
            contributions: None,
            publications: None,
            languages: None,
            fundings: None,
            subjects: None,
            issues: None,
            imprint: Default::default(),
        }
    }
}

impl Default for Doi {
    fn default() -> Doi {
        Doi {
            0: Default::default(),
        }
    }
}

impl fmt::Display for Doi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.replace(DOI_DOMAIN, ""))
    }
}

impl FromStr for Doi {
    type Err = ThothError;

    fn from_str(input: &str) -> ThothResult<Doi> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
        static ref RE: Regex = Regex::new(
            // ^    = beginning of string
            // (?:) = non-capturing group
            // i    = case-insensitive flag
            // $    = end of string
            // Matches strings of format "[[http[s]://]doi.org/]10.XXX/XXX"
            // and captures the identifier segment starting with the "10." directory indicator
            // Corresponds to database constraints although regex syntax differs slightly
            // (e.g. `;()/` do not need to be escaped here)
            r#"^(?i:(?:https?://)?doi\.org/)?(10\.\d{4,9}/[-._;()/:a-zA-Z0-9]+$)"#).unwrap();
        }
        if let Some(matches) = RE.captures(input) {
            // The 0th capture always corresponds to the entire match
            if let Some(identifier) = matches.get(1) {
                let standardised = format!("{}{}", DOI_DOMAIN, identifier.as_str());
                let doi: Doi = Doi { 0: standardised };
                Ok(doi)
            } else {
                Err(ThothError::IdentifierParseError(
                    input.to_string(),
                    "DOI".to_string(),
                ))
            }
        } else {
            Err(ThothError::IdentifierParseError(
                input.to_string(),
                "DOI".to_string(),
            ))
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
fn test_workfield_default() {
    let workfield: WorkField = Default::default();
    assert_eq!(workfield, WorkField::FullTitle);
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
fn test_workfield_display() {
    assert_eq!(format!("{}", WorkField::WorkId), "ID");
    assert_eq!(format!("{}", WorkField::WorkType), "Type");
    assert_eq!(format!("{}", WorkField::WorkStatus), "WorkStatus");
    assert_eq!(format!("{}", WorkField::FullTitle), "Title");
    assert_eq!(format!("{}", WorkField::Title), "ShortTitle");
    assert_eq!(format!("{}", WorkField::Subtitle), "Subtitle");
    assert_eq!(format!("{}", WorkField::Reference), "Reference");
    assert_eq!(format!("{}", WorkField::Edition), "Edition");
    assert_eq!(format!("{}", WorkField::Doi), "DOI");
    assert_eq!(format!("{}", WorkField::PublicationDate), "PublicationDate");
    assert_eq!(format!("{}", WorkField::Place), "Place");
    assert_eq!(format!("{}", WorkField::Width), "Width");
    assert_eq!(format!("{}", WorkField::Height), "Height");
    assert_eq!(format!("{}", WorkField::PageCount), "PageCount");
    assert_eq!(format!("{}", WorkField::PageBreakdown), "PageBreakdown");
    assert_eq!(format!("{}", WorkField::ImageCount), "ImageCount");
    assert_eq!(format!("{}", WorkField::TableCount), "TableCount");
    assert_eq!(format!("{}", WorkField::AudioCount), "AudioCount");
    assert_eq!(format!("{}", WorkField::VideoCount), "VideoCount");
    assert_eq!(format!("{}", WorkField::License), "License");
    assert_eq!(format!("{}", WorkField::CopyrightHolder), "CopyrightHolder");
    assert_eq!(format!("{}", WorkField::LandingPage), "LandingPage");
    assert_eq!(format!("{}", WorkField::Lccn), "LCCN");
    assert_eq!(format!("{}", WorkField::Oclc), "OCLC");
    assert_eq!(format!("{}", WorkField::ShortAbstract), "ShortAbstract");
    assert_eq!(format!("{}", WorkField::LongAbstract), "LongAbstract");
    assert_eq!(format!("{}", WorkField::GeneralNote), "GeneralNote");
    assert_eq!(format!("{}", WorkField::Toc), "TOC");
    assert_eq!(format!("{}", WorkField::CoverUrl), "CoverURL");
    assert_eq!(format!("{}", WorkField::CoverCaption), "CoverCaption");
    assert_eq!(format!("{}", WorkField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", WorkField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_worktype_fromstr() {
    use std::str::FromStr;
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
    use std::str::FromStr;
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

#[test]
fn test_workfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(WorkField::from_str("ID").unwrap(), WorkField::WorkId);
    assert_eq!(WorkField::from_str("Type").unwrap(), WorkField::WorkType);
    assert_eq!(
        WorkField::from_str("WorkStatus").unwrap(),
        WorkField::WorkStatus
    );
    assert_eq!(WorkField::from_str("Title").unwrap(), WorkField::FullTitle);
    assert_eq!(WorkField::from_str("ShortTitle").unwrap(), WorkField::Title);
    assert_eq!(
        WorkField::from_str("Subtitle").unwrap(),
        WorkField::Subtitle
    );
    assert_eq!(
        WorkField::from_str("Reference").unwrap(),
        WorkField::Reference
    );
    assert_eq!(WorkField::from_str("Edition").unwrap(), WorkField::Edition);
    assert_eq!(WorkField::from_str("DOI").unwrap(), WorkField::Doi);
    assert_eq!(
        WorkField::from_str("PublicationDate").unwrap(),
        WorkField::PublicationDate
    );
    assert_eq!(WorkField::from_str("Place").unwrap(), WorkField::Place);
    assert_eq!(WorkField::from_str("Width").unwrap(), WorkField::Width);
    assert_eq!(WorkField::from_str("Height").unwrap(), WorkField::Height);
    assert_eq!(
        WorkField::from_str("PageCount").unwrap(),
        WorkField::PageCount
    );
    assert_eq!(
        WorkField::from_str("PageBreakdown").unwrap(),
        WorkField::PageBreakdown
    );
    assert_eq!(
        WorkField::from_str("ImageCount").unwrap(),
        WorkField::ImageCount
    );
    assert_eq!(
        WorkField::from_str("TableCount").unwrap(),
        WorkField::TableCount
    );
    assert_eq!(
        WorkField::from_str("AudioCount").unwrap(),
        WorkField::AudioCount
    );
    assert_eq!(
        WorkField::from_str("VideoCount").unwrap(),
        WorkField::VideoCount
    );
    assert_eq!(WorkField::from_str("License").unwrap(), WorkField::License);
    assert_eq!(
        WorkField::from_str("CopyrightHolder").unwrap(),
        WorkField::CopyrightHolder
    );
    assert_eq!(
        WorkField::from_str("LandingPage").unwrap(),
        WorkField::LandingPage
    );
    assert_eq!(WorkField::from_str("LCCN").unwrap(), WorkField::Lccn);
    assert_eq!(WorkField::from_str("OCLC").unwrap(), WorkField::Oclc);
    assert_eq!(
        WorkField::from_str("ShortAbstract").unwrap(),
        WorkField::ShortAbstract
    );
    assert_eq!(
        WorkField::from_str("LongAbstract").unwrap(),
        WorkField::LongAbstract
    );
    assert_eq!(
        WorkField::from_str("GeneralNote").unwrap(),
        WorkField::GeneralNote
    );
    assert_eq!(WorkField::from_str("TOC").unwrap(), WorkField::Toc);
    assert_eq!(
        WorkField::from_str("CoverURL").unwrap(),
        WorkField::CoverUrl
    );
    assert_eq!(
        WorkField::from_str("CoverCaption").unwrap(),
        WorkField::CoverCaption
    );
    assert_eq!(
        WorkField::from_str("CreatedAt").unwrap(),
        WorkField::CreatedAt
    );
    assert_eq!(
        WorkField::from_str("UpdatedAt").unwrap(),
        WorkField::UpdatedAt
    );
    assert!(WorkField::from_str("WorkID").is_err());
    assert!(WorkField::from_str("Contributors").is_err());
    assert!(WorkField::from_str("Publisher").is_err());
}

#[test]
fn test_doi_default() {
    let doi: Doi = Default::default();
    assert_eq!(doi, Doi { 0: "".to_string() });
}

#[test]
fn test_doi_display() {
    let doi = Doi {
        0: "https://doi.org/10.12345/Test-Suffix.01".to_string(),
    };
    assert_eq!(format!("{}", doi), "10.12345/Test-Suffix.01");
}

#[test]
fn test_doi_fromstr() {
    let standardised = Doi {
        0: "https://doi.org/10.12345/Test-Suffix.01".to_string(),
    };
    assert_eq!(
        Doi::from_str("https://doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("http://doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("doi.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("HTTPS://DOI.ORG/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert_eq!(
        Doi::from_str("Https://DOI.org/10.12345/Test-Suffix.01").unwrap(),
        standardised
    );
    assert!(Doi::from_str("htts://doi.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("http://test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("test.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("//doi.org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("https://doi-org/10.12345/Test-Suffix.01").is_err());
    assert!(Doi::from_str("10.https://doi.org/12345/Test-Suffix.01").is_err());
}
