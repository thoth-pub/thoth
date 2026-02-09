use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
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
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(feature = "backend", derive(Insertable), diesel(table_name = series_history))]
pub struct NewSeriesHistory {
    pub series_id: Uuid,
    pub user_id: String,
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

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::SeriesPolicy;
#[cfg(test)]
mod tests;
