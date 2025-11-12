use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::series;
#[cfg(feature = "backend")]
use crate::schema::series_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of a series"),
    ExistingTypePath = "crate::schema::sql_types::SeriesType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum SeriesType {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "A set of collections of articles on a specific topic, published periodically"
        )
    )]
    Journal,
    #[cfg_attr(
        feature = "backend",
        db_rename = "book-series",
        graphql(description = "A set of related books, published periodically")
    )]
    #[default]
    BookSeries,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting series list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeriesField {
    #[strum(serialize = "ID")]
    SeriesId,
    SeriesType,
    #[strum(serialize = "Series")]
    #[default]
    SeriesName,
    #[strum(serialize = "ISSNPrint")]
    IssnPrint,
    #[strum(serialize = "ISSNDigital")]
    IssnDigital,
    #[strum(serialize = "SeriesURL")]
    SeriesUrl,
    CreatedAt,
    UpdatedAt,
    SeriesDescription,
    #[strum(serialize = "SeriesCFPURL")]
    SeriesCfpUrl,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: Option<String>,
    pub issn_digital: Option<String>,
    pub series_url: Option<String>,
    pub imprint_id: Uuid,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub series_description: Option<String>,
    pub series_cfp_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new periodical of publications"),
    diesel(table_name = series)
)]
pub struct NewSeries {
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: Option<String>,
    pub issn_digital: Option<String>,
    pub series_url: Option<String>,
    pub series_description: Option<String>,
    pub series_cfp_url: Option<String>,
    pub imprint_id: Uuid,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing periodical of publications"),
    diesel(table_name = series, treat_none_as_null = true)
)]
pub struct PatchSeries {
    pub series_id: Uuid,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: Option<String>,
    pub issn_digital: Option<String>,
    pub series_url: Option<String>,
    pub series_description: Option<String>,
    pub series_cfp_url: Option<String>,
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

#[cfg_attr(feature = "backend", derive(Insertable), diesel(table_name = series_history))]
pub struct NewSeriesHistory {
    pub series_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting serieses list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeriesOrderBy {
    pub field: SeriesField,
    pub direction: Direction,
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
    assert_eq!(
        format!("{}", SeriesField::SeriesDescription),
        "SeriesDescription"
    );
    assert_eq!(format!("{}", SeriesField::SeriesCfpUrl), "SeriesCFPURL");
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
        SeriesField::from_str("SeriesDescription").unwrap(),
        SeriesField::SeriesDescription
    );
    assert_eq!(
        SeriesField::from_str("SeriesCFPURL").unwrap(),
        SeriesField::SeriesCfpUrl
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
#[cfg(feature = "backend")]
pub mod crud;
