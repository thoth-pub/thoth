use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const IMPRINTS_QUERY: &str = "
    query ImprintsQuery($limit: Int, $offset: Int, $filter: String) {
        imprints(limit: $limit, offset: $offset, filter: $filter) {
            imprintId
            imprintName
            imprintUrl
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
            }
        }
        imprintCount(filter: $filter)
    }
";

graphql_query_builder! {
    ImprintsRequest,
    ImprintsRequestBody,
    Variables,
    IMPRINTS_QUERY,
    ImprintsResponseBody,
    ImprintsResponseData,
    FetchImprints,
    FetchActionImprints
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
pub struct ImprintsResponseData {
    pub imprints: Vec<Imprint>,
    pub imprint_count: i32,
}
