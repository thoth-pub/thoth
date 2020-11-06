use serde::Deserialize;
use serde::Serialize;

use super::Series;

pub const SERIESES_QUERY: &str = "
    query SeriesesQuery($limit: Int, $offset: Int, $filter: String) {
        serieses(limit: $limit, offset: $offset, filter: $filter) {
            seriesId
            seriesType
            seriesName
            issnPrint
            issnDigital
            seriesUrl
            imprint {
                imprintId
                imprintName
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                }
            }
        }
        seriesCount(filter: $filter)
    }
";

graphql_query_builder! {
    SeriesesRequest,
    SeriesesRequestBody,
    Variables,
    SERIESES_QUERY,
    SeriesesResponseBody,
    SeriesesResponseData,
    FetchSerieses,
    FetchActionSerieses
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesesResponseData {
    pub serieses: Vec<Series>,
    pub series_count: i32,
}
