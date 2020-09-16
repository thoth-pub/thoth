use serde::Deserialize;
use serde::Serialize;

const STATS_QUERY: &str = "
    {
        works(limit: 9999) { workId }
        publishers(limit: 9999) { publisherId }
        serieses(limit: 9999) { seriesId }
        contributors(limit: 9999) { contributorId }
    }
";

query_builder!{
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
pub struct StatsQueryWork {
    work_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsQueryPublisher {
    publisher_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsQuerySeries {
    series_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsQueryContributor {
    contributor_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsResponseData {
    pub works: Vec<StatsQueryWork>,
    pub publishers: Vec<StatsQueryPublisher>,
    pub serieses: Vec<StatsQuerySeries>,
    pub contributors: Vec<StatsQueryContributor>,
}

impl Default for StatsResponseData {
    fn default() -> StatsResponseData {
        StatsResponseData {
            works: vec![],
            publishers: vec![],
            serieses: vec![],
            contributors: vec![],
        }
    }
}
