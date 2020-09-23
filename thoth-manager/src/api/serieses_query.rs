use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Series;

const SERIESES_QUERY: &str = "
    {
        serieses(limit: 9999) {
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
