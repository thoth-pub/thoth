use serde::Deserialize;
use serde::Serialize;

use super::SeriesTypeDefinition;

const SERIES_TYPES_QUERY: &str = "
    {
        series_types: __type(name: \"SeriesType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    SeriesTypesRequest,
    SeriesTypesRequestBody,
    Variables,
    SERIES_TYPES_QUERY,
    SeriesTypesResponseBody,
    SeriesTypesResponseData,
    FetchSeriesTypes,
    FetchActionSeriesTypes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SeriesTypesResponseData {
    pub series_types: SeriesTypeDefinition,
}
