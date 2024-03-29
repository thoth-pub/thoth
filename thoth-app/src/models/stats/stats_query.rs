use serde::Deserialize;
use serde::Serialize;

const STATS_QUERY: &str = "
    query StatsQuery($publishers: [Uuid!]) {
        workCount(publishers: $publishers)
        bookCount(publishers: $publishers)
        chapterCount(publishers: $publishers)
        publisherCount(publishers: $publishers)
        imprintCount(publishers: $publishers)
        seriesCount(publishers: $publishers)
        contributorCount
        publicationCount(publishers: $publishers)
        institutionCount
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponseData {
    pub work_count: i32,
    pub book_count: i32,
    pub chapter_count: i32,
    pub publisher_count: i32,
    pub imprint_count: i32,
    pub series_count: i32,
    pub contributor_count: i32,
    pub publication_count: i32,
    pub institution_count: i32,
}
