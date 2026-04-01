use crate::graphql::types::inputs::Direction;
use crate::model::Doi;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::work;
#[cfg(feature = "backend")]
use crate::schema::work_history;
use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
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
    ResourcesDescription,
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

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
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
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
    pub toc: Option<String>,
    pub resources_description: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
    pub updated_at_with_relations: Timestamp,
}
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new written text that can be published"),
    diesel(table_name = work)
)]
pub struct NewWork {
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
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
    pub toc: Option<String>,
    pub resources_description: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing written text that can be published"),
    diesel(table_name = work, treat_none_as_null = true)
)]
pub struct PatchWork {
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
    pub general_note: Option<String>,
    pub bibliography_note: Option<String>,
    pub toc: Option<String>,
    pub resources_description: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub page_interval: Option<String>,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct WorkHistory {
    pub work_history_id: Uuid,
    pub work_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(diesel::Insertable), diesel(table_name = work_history))]
pub struct NewWorkHistory {
    pub work_id: Uuid,
    pub user_id: String,
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
            general_note: w.general_note,
            bibliography_note: w.bibliography_note,
            toc: w.toc,
            resources_description: w.resources_description,
            cover_url: w.cover_url,
            cover_caption: w.cover_caption,
            first_page: w.first_page,
            last_page: w.last_page,
            page_interval: w.page_interval,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::WorkPolicy;
#[cfg(test)]
mod tests;
