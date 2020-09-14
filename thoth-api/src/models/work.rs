use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::work;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Work_type")]
#[derive(Debug, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, PartialEq, Deserialize, Serialize)]
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
    pub lccn: Option<i32>,
    pub oclc: Option<i32>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "work")]
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
    pub lccn: Option<i32>,
    pub oclc: Option<i32>,
    pub short_abstract: Option<String>,
    pub long_abstract: Option<String>,
    pub general_note: Option<String>,
    pub toc: Option<String>,
    pub cover_url: Option<String>,
    pub cover_caption: Option<String>,
}
