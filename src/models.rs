use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::schema::work;

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Contribution_type"]
pub enum ContributionType {
    Author,
    Editor,
    Translator,
    Photographer,
    Ilustrator,
    #[db_rename = "foreword-by"]
    ForewordBy,
    #[db_rename = "introduction-by"]
    IntroductionBy,
    #[db_rename = "afterword-by"]
    AfterwordBy,
    #[db_rename = "preface-by"]
    PrefaceBy,
}

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
#[DieselType = "Series_type"]
pub enum SeriesType {
    Journal,
    #[db_rename = "book-series"]
    BookSeries,
}

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Publication_type"]
pub enum PublicationType {
    #[db_rename = "Paperback"]
    Paperback,
    #[db_rename = "Hardback"]
    Hardback,
    #[db_rename = "PDF"]
    PDF,
    #[db_rename = "HTML"]
    HTML,
    #[db_rename = "XML"]
    XML,
    #[db_rename = "Epub"]
    Epub,
    #[db_rename = "Mobi"]
    Mobi,
}

#[derive(Queryable)]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub publisher_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "work"]
pub struct NewWork {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub publisher_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
}

#[derive(Queryable)]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub publisher_id: Uuid,
}

#[derive(Queryable)]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[derive(Queryable)]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(Queryable)]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Queryable)]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(Queryable)]
pub struct Contribution {
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}
