use uuid::Uuid;
use crate::schema::series;
use crate::schema::issue;

#[derive(Debug, PartialEq, DbEnum)]
#[derive(juniper::GraphQLEnum)]
#[DieselType = "Series_type"]
pub enum SeriesType {
    Journal,
    #[db_rename = "book-series"]
    BookSeries,
}

#[derive(Queryable)]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "series"]
pub struct NewSeries {
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[derive(Queryable)]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "issue"]
pub struct NewIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}
