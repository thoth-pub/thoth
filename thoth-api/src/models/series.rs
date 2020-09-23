use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::issue;
#[cfg(feature = "backend")]
use crate::schema::series;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Series_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeriesType {
    Journal,
    #[cfg_attr(feature = "backend", db_rename = "book-series")]
    BookSeries,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "series")]
pub struct NewSeries {
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Issue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "issue")]
pub struct NewIssue {
    pub series_id: Uuid,
    pub work_id: Uuid,
    pub issue_ordinal: i32,
}

impl fmt::Display for SeriesType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SeriesType::Journal => write!(f, "Journal"),
            SeriesType::BookSeries => write!(f, "Book Series"),
        }
    }
}
