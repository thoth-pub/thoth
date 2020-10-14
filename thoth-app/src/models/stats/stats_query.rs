use serde::Deserialize;
use serde::Serialize;

const STATS_QUERY: &str = "
    {
        workCount
        publisherCount
        imprintCount
        seriesCount
        contributorCount
        publicationCount
    }
";

graphql_query_builder! {
    StatsRequest,
    StatsRequestBody,
    STATS_QUERY,
    StatsResponseBody,
    StatsResponseData,
    FetchStats,
    FetchActionStats
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponseData {
    pub work_count: i32,
    pub publisher_count: i32,
    pub imprint_count: i32,
    pub series_count: i32,
    pub contributor_count: i32,
    pub publication_count: i32,
}

impl Default for StatsResponseData {
    fn default() -> StatsResponseData {
        StatsResponseData {
            work_count: 0,
            publisher_count: 0,
            imprint_count: 0,
            series_count: 0,
            contributor_count: 0,
            publication_count: 0,
        }
    }
}
