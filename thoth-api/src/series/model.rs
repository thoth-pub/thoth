use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::ThothError;
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

impl Default for SeriesType {
    fn default() -> SeriesType {
        SeriesType::BookSeries
    }
}

impl fmt::Display for SeriesType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SeriesType::Journal => write!(f, "Journal"),
            SeriesType::BookSeries => write!(f, "Book Series"),
        }
    }
}

impl FromStr for SeriesType {
    type Err = ThothError;

    fn from_str(input: &str) -> Result<SeriesType, ThothError> {
        match input {
            "Journal" => Ok(SeriesType::Journal),
            "Book Series" => Ok(SeriesType::BookSeries),
            _ => Err(ThothError::InvalidSeriesType(input.to_string())),
        }
    }
}

#[test]
fn test_seriestype_default() {
    let seriestype: SeriesType = Default::default();
    assert_eq!(seriestype, SeriesType::BookSeries);
}

#[test]
fn test_seriestype_display() {
    assert_eq!(format!("{}", SeriesType::Journal), "Journal");
    assert_eq!(format!("{}", SeriesType::BookSeries), "Book Series");
}

#[test]
fn test_seriestype_fromstr() {
    assert_eq!(
        SeriesType::from_str("Journal").unwrap(),
        SeriesType::Journal
    );
    assert_eq!(
        SeriesType::from_str("Book Series").unwrap(),
        SeriesType::BookSeries
    );

    assert!(SeriesType::from_str("bookseries").is_err());
    assert!(SeriesType::from_str("Collection").is_err());
}
