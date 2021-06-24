use serde::{Deserialize, Serialize};
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::imprint::model::ImprintExtended as Imprint;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::series;
#[cfg(feature = "backend")]
use crate::schema::series_history;

#[cfg_attr(feature = "backend", derive(DbEnum, juniper::GraphQLEnum))]
#[cfg_attr(feature = "backend", DieselType = "Series_type")]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum SeriesType {
    Journal,
    #[cfg_attr(feature = "backend", db_rename = "book-series")]
    BookSeries,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting series list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeriesField {
    #[strum(serialize = "ID")]
    SeriesId,
    SeriesType,
    #[strum(serialize = "Series")]
    SeriesName,
    #[strum(serialize = "ISSNPrint")]
    IssnPrint,
    #[strum(serialize = "ISSNDigital")]
    IssnDigital,
    #[strum(serialize = "SeriesURL")]
    SeriesUrl,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesExtended {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub updated_at: Timestamp,
    pub imprint: Imprint,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "series"
)]
pub struct NewSeries {
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "series"
)]
pub struct PatchSeries {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct SeriesHistory {
    pub series_history_id: Uuid,
    pub series_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Insertable), table_name = "series_history")]
pub struct NewSeriesHistory {
    pub series_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting seriess list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SeriesOrderBy {
    pub field: SeriesField,
    pub direction: Direction,
}

impl Default for SeriesType {
    fn default() -> SeriesType {
        SeriesType::BookSeries
    }
}

impl Default for SeriesField {
    fn default() -> Self {
        SeriesField::SeriesName
    }
}

impl fmt::Display for SeriesExtended {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}, {})",
            self.series_name, self.issn_print, self.issn_digital
        )
    }
}

#[test]
fn test_seriestype_default() {
    let seriestype: SeriesType = Default::default();
    assert_eq!(seriestype, SeriesType::BookSeries);
}

#[test]
fn test_seriesfield_default() {
    let seriesfield: SeriesField = Default::default();
    assert_eq!(seriesfield, SeriesField::SeriesName);
}

#[test]
fn test_seriestype_display() {
    assert_eq!(format!("{}", SeriesType::Journal), "Journal");
    assert_eq!(format!("{}", SeriesType::BookSeries), "Book Series");
}

#[test]
fn test_seriesfield_display() {
    assert_eq!(format!("{}", SeriesField::SeriesId), "ID");
    assert_eq!(format!("{}", SeriesField::SeriesType), "SeriesType");
    assert_eq!(format!("{}", SeriesField::SeriesName), "Series");
    assert_eq!(format!("{}", SeriesField::IssnPrint), "ISSNPrint");
    assert_eq!(format!("{}", SeriesField::IssnDigital), "ISSNDigital");
    assert_eq!(format!("{}", SeriesField::SeriesUrl), "SeriesURL");
    assert_eq!(format!("{}", SeriesField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", SeriesField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_seriestype_fromstr() {
    use std::str::FromStr;
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

#[test]
fn test_seriesfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(SeriesField::from_str("ID").unwrap(), SeriesField::SeriesId);
    assert_eq!(
        SeriesField::from_str("SeriesType").unwrap(),
        SeriesField::SeriesType
    );
    assert_eq!(
        SeriesField::from_str("Series").unwrap(),
        SeriesField::SeriesName
    );
    assert_eq!(
        SeriesField::from_str("ISSNPrint").unwrap(),
        SeriesField::IssnPrint
    );
    assert_eq!(
        SeriesField::from_str("ISSNDigital").unwrap(),
        SeriesField::IssnDigital
    );
    assert_eq!(
        SeriesField::from_str("SeriesURL").unwrap(),
        SeriesField::SeriesUrl
    );
    assert_eq!(
        SeriesField::from_str("CreatedAt").unwrap(),
        SeriesField::CreatedAt
    );
    assert_eq!(
        SeriesField::from_str("UpdatedAt").unwrap(),
        SeriesField::UpdatedAt
    );
    assert!(SeriesField::from_str("SeriesID").is_err());
    assert!(SeriesField::from_str("Publisher").is_err());
    assert!(SeriesField::from_str("Issues").is_err());
}
