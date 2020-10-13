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
        }
    }
";

query_builder! {
    SeriesesRequest,
    SeriesesRequestBody,
    SERIESES_QUERY,
    SeriesesResponseBody,
    SeriesesResponseData,
    FetchSerieses,
    FetchActionSerieses
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeriesesResponseData {
    pub serieses: Vec<Series>,
}

impl Default for SeriesesResponseData {
    fn default() -> SeriesesResponseData {
        SeriesesResponseData { serieses: vec![] }
    }
}
