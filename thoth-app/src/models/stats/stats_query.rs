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
    Variables,
    STATS_QUERY,
    StatsResponseBody,
    StatsResponseData,
    FetchStats,
    FetchActionStats
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponseData {
    pub work_count: i32,
    pub publisher_count: i32,
    pub imprint_count: i32,
    pub series_count: i32,
    pub contributor_count: i32,
    pub publication_count: i32,
}
