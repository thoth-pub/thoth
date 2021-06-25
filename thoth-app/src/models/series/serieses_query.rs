use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesOrderBy;
use thoth_api::series::model::SeriesWithImprint;

pub const SERIESES_QUERY: &str = "
    query SeriesesQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!], $order: SeriesOrderBy) {
        serieses(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers, order: $order) {
            seriesId
            seriesType
            seriesName
            issnPrint
            issnDigital
            seriesUrl
            updatedAt
            imprint {
                imprintId
                imprintName
                updatedAt
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                    createdAt
                    updatedAt
                }
            }
        }
        seriesCount(filter: $filter, publishers: $publishers)
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
    pub order: Option<SeriesOrderBy>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesesResponseData {
    pub serieses: Vec<SeriesWithImprint>,
    pub series_count: i32,
}
