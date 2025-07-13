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
use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::Display;
use strum::EnumString;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::title::Title;
use super::r#abstract::Abstract;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of a work"),
    ExistingTypePath = "crate::schema::sql_types::WorkType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum WorkType {
    #[cfg_attr(
        feature = "backend",
        db_rename = "book-chapter",
        graphql(description = "Section of a larger parent work")
    )]
    BookChapter,
    #[default]
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Long-form work on a single theme, by a small number of authors")
    )]
    Monograph,
    #[cfg_attr(
        feature = "backend",
        db_rename = "edited-book",
        graphql(description = "Collection of short works by different authors on a single theme")
    )]
    EditedBook,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Work used for educational purposes")
    )]
    Textbook,
    #[cfg_attr(
        feature = "backend",
        db_rename = "journal-issue",
        graphql(
            description = "Single publication within a series of collections of related articles"
        )
    )]
    JournalIssue,
    #[cfg_attr(
        feature = "backend",
        db_rename = "book-set",
        graphql(description = "Group of volumes published together forming a single work")
    )]
    BookSet,
}

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Publication status of a work throughout its lifecycle. For a visual representation of the workflow, refer to the work status flowchart https://github.com/thoth-pub/thoth/wiki/Thoth_Works#work-status-flowchart"
    ),
    ExistingTypePath = "crate::schema::sql_types::WorkStatus"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum WorkStatus {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work is in progress and is expected to be published. This is the typical status for a work that has not yet been released but is planned for publication."
        )
    )]
    #[default]
    Forthcoming,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work is published and currently available. This status indicates that the work is officially released."
        )
    )]
    Active,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work has been withdrawn from publication and will be removed from all distribution channels. This status indicates that the work is no longer available for sale or distribution and will no longer be accessible."
        )
    )]
    Withdrawn,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work has been replaced by a new edition, with the previous edition now considered outdated. The two editions should be linked using a `WorkRelation` of type `REPLACES`/`IS_REPLACED_BY`."
        )
    )]
    Superseded,
    #[cfg_attr(
        feature = "backend",
        db_rename = "postponed-indefinitely",
        graphql(
            description = "The work's release has been delayed indefinitely. It may be resumed at a later time, but currently, no publication date is set."
        )
    )]
    PostponedIndefinitely,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The work has been permanently cancelled and will not be published."
        )
    )]
    Cancelled,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting works list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkField {
    #[strum(serialize = "ID")]
    WorkId,
    #[strum(serialize = "Type")]
    WorkType,
    WorkStatus,
    #[strum(serialize = "Title")]
    #[default]
    FullTitle,
    #[strum(serialize = "ShortTitle")]
    Title,
    Subtitle,
    Reference,
    Edition,
    #[strum(serialize = "DOI")]
    Doi,
    PublicationDate,
    WithdrawnDate,
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
    BibliographyNote,
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
    UpdatedAtWithRelations,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub withdrawn_date: Option<NaiveDate>,
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
    // pub short_abstract: Option<String>,
    // pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
    pub updated_at_with_relations: Timestamp,
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
    pub publication_date: Option<NaiveDate>,
    pub withdrawn_date: Option<NaiveDate>,
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
    pub bibliography_note: Option<String>,
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
    pub titles: Option<Vec<Title>>,
    pub abstracts: Option<Vec<Abstract>>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new written text that can be published"),
    diesel(table_name = work)
)]
pub struct NewWork {
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    // pub full_title: String,
    // pub title: String,
    // pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub withdrawn_date: Option<NaiveDate>,
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
    // pub short_abstract: Option<String>,
    // pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
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
    graphql(description = "Set of values required to update an existing written text that can be published"),
    diesel(table_name = work, treat_none_as_null = true)
)]
pub struct PatchWork {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    // pub full_title: String,
    // pub title: String,
    // pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: Option<i32>,
    pub imprint_id: Uuid,
    pub doi: Option<Doi>,
    pub publication_date: Option<NaiveDate>,
    pub withdrawn_date: Option<NaiveDate>,
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
    // pub short_abstract: Option<String>,
    // pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
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

#[cfg_attr(feature = "backend", derive(Insertable), diesel(table_name = work_history))]
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

pub trait WorkProperties {
    // fn title(&self) -> &str;
    // fn subtitle(&self) -> Option<&str>;
    fn work_status(&self) -> &WorkStatus;
    fn publication_date(&self) -> &Option<NaiveDate>;
    fn withdrawn_date(&self) -> &Option<NaiveDate>;
    fn first_page(&self) -> Option<&str>;
    fn last_page(&self) -> Option<&str>;

    fn compile_page_interval(&self) -> Option<String> {
        self.first_page()
            .zip(self.last_page())
            .map(|(first, last)| format!("{first}–{last}"))
    }

    fn is_published(&self) -> bool {
        matches!(
            self.work_status(),
            WorkStatus::Active | WorkStatus::Withdrawn | WorkStatus::Superseded
        )
    }

    fn is_active(&self) -> bool {
        matches!(self.work_status(), WorkStatus::Active)
    }

    fn is_out_of_print(&self) -> bool {
        matches!(
            self.work_status(),
            WorkStatus::Withdrawn | WorkStatus::Superseded
        )
    }

    fn validate(&self) -> ThothResult<()> {
        match (
            self.is_published(),
            self.publication_date(),
            self.is_out_of_print(),
            self.withdrawn_date(),
        ) {
            (true, None, _, _) => Err(ThothError::PublicationDateError),
            (_, _, false, Some(_)) => Err(ThothError::WithdrawnDateError),
            (_, _, true, None) => Err(ThothError::NoWithdrawnDateError),
            (_, Some(publication), _, Some(withdrawn)) if withdrawn < publication => {
                Err(ThothError::WithdrawnDateBeforePublicationDateError)
            }
            _ => Ok(()),
        }
    }
}

macro_rules! work_properties {
    ($t:ty) => {
        impl WorkProperties for $t {
            fn work_status(&self) -> &WorkStatus {
                &self.work_status
            }
            fn publication_date(&self) -> &Option<NaiveDate> {
                &self.publication_date
            }
            fn withdrawn_date(&self) -> &Option<NaiveDate> {
                &self.withdrawn_date
            }
            fn first_page(&self) -> Option<&str> {
                self.first_page.as_deref()
            }
            fn last_page(&self) -> Option<&str> {
                self.last_page.as_deref()
            }
        }
    };
}

work_properties!(Work);
work_properties!(NewWork);
work_properties!(PatchWork);
work_properties!(WorkWithRelations);

impl WorkWithRelations {
    pub fn publisher(&self) -> String {
        self.imprint
            .publisher
            .publisher_shortname
            .as_ref()
            .map_or_else(
                || self.imprint.publisher.publisher_name.to_string(),
                |short_name| short_name.to_string(),
            )
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn compile_fulltitle(&self) -> String {
        self.subtitle().map_or_else(
            || self.title().to_string(),
            |subtitle| {
                let title = self.title();
                if title.ends_with('?')
                    || title.ends_with('!')
                    || title.ends_with(':')
                    || title.ends_with('.')
                {
                    format!("{} {}", title, subtitle)
                } else {
                    format!("{}: {}", title, subtitle)
                }
            },
        )
    }
}

impl From<Work> for PatchWork {
    fn from(w: Work) -> Self {
        Self {
            work_id: w.work_id,
            work_type: w.work_type,
            work_status: w.work_status,
            reference: w.reference,
            edition: w.edition,
            imprint_id: w.imprint_id,
            doi: w.doi,
            publication_date: w.publication_date,
            withdrawn_date: w.withdrawn_date,
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
            // short_abstract: w.short_abstract,
            // long_abstract: w.long_abstract,
            general_note: w.general_note,
            bibliography_note: w.bibliography_note,
            toc: w.toc,
            cover_url: w.cover_url,
            cover_caption: w.cover_caption,
            first_page: w.first_page,
            last_page: w.last_page,
            page_interval: w.page_interval,
        }
    }
}

impl fmt::Display for Work {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.doi {
            Some(doi) => write!(f, "{} - {}", self.work_id, doi),
            None => write!(f, "{}", self.work_id),
        }
    }
}

impl fmt::Display for WorkWithRelations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.doi {
            Some(doi) => write!(f, "{} - {}", self.full_title, doi),
            None => write!(f, "{}", self.full_title),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_work() -> Work {
        use std::str::FromStr;
        Work {
            work_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
            work_type: WorkType::Monograph,
            work_status: WorkStatus::Active,
            reference: None,
            edition: Some(1),
            imprint_id: Uuid::parse_str("00000000-0000-0000-BBBB-000000000002").unwrap(),
            doi: Some(Doi::from_str("https://doi.org/10.00001/BOOK.0001").unwrap()),
            publication_date: NaiveDate::from_ymd_opt(1999, 12, 31),
            withdrawn_date: None,
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
            // short_abstract: Some("Short abstract".to_string()),
            // long_abstract: Some("Long abstract".to_string()),
            general_note: None,
            bibliography_note: None,
            toc: None,
            cover_url: Some("https://book.cover/image".to_string()),
            cover_caption: None,
            created_at: Default::default(),
            updated_at: Default::default(),
            first_page: None,
            last_page: None,
            page_interval: None,
            updated_at_with_relations: Default::default(),
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
        assert_eq!(workstatus, WorkStatus::Forthcoming);
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
        assert_eq!(format!("{}", WorkStatus::Withdrawn), "Withdrawn");
        assert_eq!(format!("{}", WorkStatus::Superseded), "Superseded");
    }

    #[test]
    fn test_workfield_display() {
        assert_eq!(format!("{}", WorkField::WorkId), "ID");
        assert_eq!(format!("{}", WorkField::WorkType), "Type");
        assert_eq!(format!("{}", WorkField::WorkStatus), "WorkStatus");
        // assert_eq!(format!("{}", WorkField::FullTitle), "Title");
        // assert_eq!(format!("{}", WorkField::Title), "ShortTitle");
        // assert_eq!(format!("{}", WorkField::Subtitle), "Subtitle");
        assert_eq!(format!("{}", WorkField::Reference), "Reference");
        assert_eq!(format!("{}", WorkField::Edition), "Edition");
        assert_eq!(format!("{}", WorkField::Doi), "DOI");
        assert_eq!(format!("{}", WorkField::PublicationDate), "PublicationDate");
        assert_eq!(format!("{}", WorkField::WithdrawnDate), "WithdrawnDate");
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
        assert_eq!(
            format!("{}", WorkField::BibliographyNote),
            "BibliographyNote"
        );
        assert_eq!(format!("{}", WorkField::Toc), "TOC");
        assert_eq!(format!("{}", WorkField::CoverUrl), "CoverURL");
        assert_eq!(format!("{}", WorkField::CoverCaption), "CoverCaption");
        assert_eq!(format!("{}", WorkField::CreatedAt), "CreatedAt");
        assert_eq!(format!("{}", WorkField::UpdatedAt), "UpdatedAt");
        assert_eq!(
            format!("{}", WorkField::UpdatedAtWithRelations),
            "UpdatedAtWithRelations"
        );
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
            WorkStatus::from_str("Withdrawn").unwrap(),
            WorkStatus::Withdrawn
        );
        assert_eq!(
            WorkStatus::from_str("Superseded").unwrap(),
            WorkStatus::Superseded
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
        assert_eq!(
            WorkField::from_str("WithdrawnDate").unwrap(),
            WorkField::WithdrawnDate
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
        assert_eq!(
            WorkField::from_str("BibliographyNote").unwrap(),
            WorkField::BibliographyNote
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
        assert_eq!(
            WorkField::from_str("UpdatedAtWithRelations").unwrap(),
            WorkField::UpdatedAtWithRelations
        );
        assert!(WorkField::from_str("WorkID").is_err());
        assert!(WorkField::from_str("Contributors").is_err());
        assert!(WorkField::from_str("Publisher").is_err());
    }

    #[test]
    fn test_work_into_patchwork() {
        let work = test_work();
        let patch_work: PatchWork = work.clone().into();

        macro_rules! assert_fields_eq {
            ($($field:ident),+) => {
                $(
                    assert_eq!(work.$field, patch_work.$field);
                )+
            };
        }
        assert_fields_eq!(
            work_id,
            work_type,
            work_status,
            reference,
            edition,
            imprint_id,
            doi,
            publication_date,
            withdrawn_date,
            place,
            page_count,
            page_breakdown,
            image_count,
            table_count,
            audio_count,
            video_count,
            license,
            copyright_holder,
            landing_page,
            lccn,
            oclc,
            // short_abstract,
            // long_abstract,
            general_note,
            bibliography_note,
            toc,
            cover_url,
            cover_caption,
            first_page,
            last_page,
            page_interval
        );
    }

    #[test]
    fn test_compile_page_interval() {
        let mut work = test_work();
        assert!(work.compile_page_interval().is_none());

        work.first_page = Some("1".to_string());
        work.last_page = Some("10".to_string());
        assert_eq!(work.compile_page_interval(), Some("1–10".to_string()));
    }

    #[test]
    fn test_is_published() {
        let mut work = test_work();

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_published());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_published());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_published());

        work.work_status = WorkStatus::Active;
        assert!(work.is_published());
        work.work_status = WorkStatus::Withdrawn;
        assert!(work.is_published());
        work.work_status = WorkStatus::Superseded;
        assert!(work.is_published());
    }

    #[test]
    fn test_is_out_of_print() {
        let mut work = test_work();

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_out_of_print());
        work.work_status = WorkStatus::Active;
        assert!(!work.is_out_of_print());

        work.work_status = WorkStatus::Withdrawn;
        assert!(work.is_out_of_print());
        work.work_status = WorkStatus::Superseded;
        assert!(work.is_out_of_print());
    }

    #[test]
    fn test_is_active() {
        let mut work = test_work();
        assert!(work.is_active());

        work.work_status = WorkStatus::Forthcoming;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Cancelled;
        assert!(!work.is_active());
        work.work_status = WorkStatus::PostponedIndefinitely;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Withdrawn;
        assert!(!work.is_active());
        work.work_status = WorkStatus::Superseded;
        assert!(!work.is_active());
    }

    #[test]
    fn test_validate_fails_when_published_without_publication_date() {
        let mut work = test_work();
        work.work_status = WorkStatus::Active;
        work.publication_date = None;

        assert_eq!(work.validate(), Err(ThothError::PublicationDateError));
    }

    #[test]
    fn test_validate_fails_when_published_with_withdrawn_date() {
        let mut work = test_work();
        work.work_status = WorkStatus::Active;
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());

        assert_eq!(work.validate(), Err(ThothError::WithdrawnDateError));
    }

    #[test]
    fn test_validate_fails_when_out_of_print_without_withdrawn_date() {
        let mut work = test_work();
        work.work_status = WorkStatus::Withdrawn;
        work.withdrawn_date = None;

        assert_eq!(work.validate(), Err(ThothError::NoWithdrawnDateError));
        work.work_status = WorkStatus::Superseded;
        assert_eq!(work.validate(), Err(ThothError::NoWithdrawnDateError));
    }

    #[test]
    fn test_validate_fails_when_withdrawn_date_before_publication_date() {
        let mut work = test_work();
        work.work_status = WorkStatus::Withdrawn;
        work.publication_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2019, 12, 31).unwrap());

        assert_eq!(
            work.validate(),
            Err(ThothError::WithdrawnDateBeforePublicationDateError)
        );
    }

    #[test]
    fn test_validate_succeeds() {
        let mut work = test_work();
        work.work_status = WorkStatus::Withdrawn;
        work.publication_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        work.withdrawn_date = Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());

        assert_eq!(work.validate(), Ok(()));
    }
}

#[cfg(feature = "backend")]
pub mod crud;
