use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::contribution::Contribution;
use crate::model::funding::FundingWithInstitution;
use crate::model::imprint::ImprintWithPublisher;
use crate::model::issue::IssueWithSeries;
use crate::model::language::Language;
use crate::model::publication::Publication;
use crate::model::reference::Reference;
use crate::model::subject::Subject;
use crate::model::work_relation::WorkRelationWithRelatedWork;
use crate::model::Doi;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::work;
#[cfg(feature = "backend")]
use crate::schema::work_history;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Work_type")]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
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
    FirstPage,
    LastPage,
    PageInterval,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: Option<String>,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkWithRelations {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub doi: Option<Doi>,
    pub publication_date: Option<String>,
    pub place: Option<String>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: Option<String>,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub updated_at: Timestamp,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
    pub contributions: Option<Vec<Contribution>>,
    pub publications: Option<Vec<Publication>>,
    pub languages: Option<Vec<Language>>,
    pub fundings: Option<Vec<FundingWithInstitution>>,
    pub subjects: Option<Vec<Subject>>,
    pub issues: Option<Vec<IssueWithSeries>>,
    pub imprint: ImprintWithPublisher,
    pub relations: Option<Vec<WorkRelationWithRelatedWork>>,
    pub references: Option<Vec<Reference>>,
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
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: Option<String>,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
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
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub place: Option<String>,
    pub page_count: Option<i32>,
    pub page_breakdown: Option<String>,
    pub image_count: Option<i32>,
    pub table_count: Option<i32>,
    pub audio_count: Option<i32>,
    pub video_count: Option<i32>,
    pub license: Option<String>,
    pub copyright_holder: Option<String>,
    pub landing_page: Option<String>,
    pub lccn: Option<String>,
    pub oclc: Option<String>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct WorkHistory {
    pub work_history_id: Uuid,
    pub work_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkOrderBy {
    pub field: WorkField,
    pub direction: Direction,
}

impl Work {
    pub fn compile_fulltitle(&self) -> String {
        if let Some(subtitle) = &self.subtitle.clone() {
            format!("{}: {}", self.title, subtitle)
        } else {
            self.title.to_string()
        }
    }
}

impl WorkWithRelations {
    pub fn compile_fulltitle(&self) -> String {
        if let Some(subtitle) = &self.subtitle.clone() {
            format!("{}: {}", self.title, subtitle)
        } else {
            self.title.to_string()
        }
    }

    pub fn compile_page_interval(&self) -> Option<String> {
        if let (Some(first), Some(last)) = (&self.first_page.clone(), &self.last_page.clone()) {
            Some(format!("{first}–{last}"))
        } else {
            None
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

impl From<Work> for PatchWork {
    fn from(w: Work) -> Self {
        Self {
            work_id: w.work_id,
            work_type: w.work_type,
            work_status: w.work_status,
            full_title: w.full_title,
            title: w.title,
            subtitle: w.subtitle,
            reference: w.reference,
            edition: w.edition,
            imprint_id: w.imprint_id,
            doi: w.doi,
            publication_date: w.publication_date,
            place: w.place,
            page_count: w.page_count,
            page_breakdown: w.page_breakdown,
            image_count: w.image_count,
            table_count: w.table_count,
            audio_count: w.audio_count,
            video_count: w.video_count,
            license: w.license,
            copyright_holder: w.copyright_holder,
            landing_page: w.landing_page,
            lccn: w.lccn,
            oclc: w.oclc,
            short_abstract: w.short_abstract,
            long_abstract: w.long_abstract,
            general_note: w.general_note,
            toc: w.toc,
            cover_url: w.cover_url,
            cover_caption: w.cover_caption,
            first_page: w.first_page,
            last_page: w.last_page,
            page_interval: w.page_interval,
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

impl fmt::Display for Work {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(doi) = &self.doi {
            write!(f, "{} - {}", &self.full_title, doi)
        } else {
            write!(f, "{}", self.full_title)
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
    assert_eq!(format!("{}", WorkField::PageCount), "PageCount");
    assert_eq!(format!("{}", WorkField::PageBreakdown), "PageBreakdown");
    assert_eq!(format!("{}", WorkField::FirstPage), "FirstPage");
    assert_eq!(format!("{}", WorkField::LastPage), "LastPage");
    assert_eq!(format!("{}", WorkField::PageInterval), "PageInterval");
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
    assert_eq!(
        WorkField::from_str("PageCount").unwrap(),
        WorkField::PageCount
    );
    assert_eq!(
        WorkField::from_str("PageBreakdown").unwrap(),
        WorkField::PageBreakdown
    );
    assert_eq!(
        WorkField::from_str("FirstPage").unwrap(),
        WorkField::FirstPage
    );
    assert_eq!(
        WorkField::from_str("LastPage").unwrap(),
        WorkField::LastPage
    );
    assert_eq!(
        WorkField::from_str("PageInterval").unwrap(),
        WorkField::PageInterval
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
fn test_work_into_patchwork() {
    use std::str::FromStr;

    let work = Work {
        work_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
        work_type: WorkType::Monograph,
        work_status: WorkStatus::Active,
        full_title: "Some title".to_string(),
        title: "Some title".to_string(),
        subtitle: None,
        reference: None,
        edition: Some(1),
        imprint_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
        doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
        publication_date: Some(chrono::NaiveDate::from_ymd(1999, 12, 31)),
        place: Some("León, Spain".to_string()),
        page_count: Some(123),
        page_breakdown: None,
        image_count: Some(22),
        table_count: Some(3),
        audio_count: None,
        video_count: None,
        license: Some("https://creativecommons.org/licenses/by/4.0/".to_string()),
        copyright_holder: Some("Author1".to_string()),
        landing_page: Some("https://book.page".to_string()),
        lccn: None,
        oclc: None,
        short_abstract: Some("Short abstract".to_string()),
        long_abstract: Some("Long abstract".to_string()),
        general_note: None,
        toc: None,
        cover_url: Some("https://book.cover/image".to_string()),
        cover_caption: None,
        created_at: Default::default(),
        updated_at: Default::default(),
        first_page: None,
        last_page: None,
        page_interval: None,
    };
    let patch_work: PatchWork = work.clone().into();

    assert_eq!(work.work_id, patch_work.work_id);
    assert_eq!(work.work_type, patch_work.work_type);
    assert_eq!(work.work_status, patch_work.work_status);
    assert_eq!(work.full_title, patch_work.full_title);
    assert_eq!(work.title, patch_work.title);
    assert_eq!(work.subtitle, patch_work.subtitle);
    assert_eq!(work.reference, patch_work.reference);
    assert_eq!(work.edition, patch_work.edition);
    assert_eq!(work.imprint_id, patch_work.imprint_id);
    assert_eq!(work.doi, patch_work.doi);
    assert_eq!(work.publication_date, patch_work.publication_date);
    assert_eq!(work.place, patch_work.place);
    assert_eq!(work.page_count, patch_work.page_count);
    assert_eq!(work.page_breakdown, patch_work.page_breakdown);
    assert_eq!(work.image_count, patch_work.image_count);
    assert_eq!(work.table_count, patch_work.table_count);
    assert_eq!(work.audio_count, patch_work.audio_count);
    assert_eq!(work.video_count, patch_work.video_count);
    assert_eq!(work.license, patch_work.license);
    assert_eq!(work.copyright_holder, patch_work.copyright_holder);
    assert_eq!(work.landing_page, patch_work.landing_page);
    assert_eq!(work.lccn, patch_work.lccn);
    assert_eq!(work.oclc, patch_work.oclc);
    assert_eq!(work.short_abstract, patch_work.short_abstract);
    assert_eq!(work.long_abstract, patch_work.long_abstract);
    assert_eq!(work.general_note, patch_work.general_note);
    assert_eq!(work.toc, patch_work.toc);
    assert_eq!(work.cover_url, patch_work.cover_url);
    assert_eq!(work.cover_caption, patch_work.cover_caption);
    assert_eq!(work.first_page, patch_work.first_page);
    assert_eq!(work.last_page, patch_work.last_page);
    assert_eq!(work.page_interval, patch_work.page_interval);
}

#[cfg(feature = "backend")]
pub mod crud;
