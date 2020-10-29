use serde::Deserialize;
use serde::Serialize;

use super::Series;

pub const SERIES_QUERY: &str = "
    query SeriesQuery($seriesId: Uuid!) {
        series(seriesId: $seriesId) {
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
    SeriesRequest,
    SeriesRequestBody,
    Variables,
    SERIES_QUERY,
    SeriesResponseBody,
    SeriesResponseData,
    FetchSeries,
    FetchActionSeries
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub series_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SeriesResponseData {
    pub series: Option<Series>,
}
