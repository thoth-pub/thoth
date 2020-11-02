use serde::Deserialize;
use serde::Serialize;

use super::Series;

pub const SERIESES_QUERY: &str = "
    query SeriesesQuery($filter: String) {
        serieses(limit: 9999, filter: $filter) {
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
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SeriesesResponseData {
    pub serieses: Vec<Series>,
}
