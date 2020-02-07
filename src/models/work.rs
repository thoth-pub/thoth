use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::schema::work;

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Work_type"]
pub enum WorkType {
    #[db_rename = "book-chapter"]
    BookChapter,
    Monograph,
    #[db_rename = "edited-book"]
    EditedBook,
    Textbook,
    #[db_rename = "journal-issue"]
    JournalIssue,
    #[db_rename = "book-set"]
    BookSet,
}

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Work_status"]
pub enum WorkStatus {
    Unspecified,
    Cancelled,
    Forthcoming,
    #[db_rename = "postponed-indefinitely"]
    PostponedIndefinitely,
    Active,
    #[db_rename = "no-longer-our-product"]
    NoLongerOurProduct,
    #[db_rename = "out-of-stock-indefinitely"]
    OutOfStockIndefinitely,
    #[db_rename = "out-of-print"]
    OutOfPrint,
    Inactive,
    Unknown,
    Remaindered,
    #[db_rename = "withdrawn-from-sale"]
    WithdrawnFromSale,
    Recalled,
}

#[derive(Queryable)]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub publisher_id: Uuid,
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

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "work"]
pub struct NewWork {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub work_status: WorkStatus,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub reference: Option<String>,
    pub edition: i32,
    pub publisher_id: Uuid,
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
