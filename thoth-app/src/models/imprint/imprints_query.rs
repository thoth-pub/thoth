use serde::Deserialize;
use serde::Serialize;
use thoth_api::graphql::utils::GenericOrderBy;

use super::Imprint;

const IMPRINTS_QUERY: &str = "
    query ImprintsQuery($limit: Int, $offset: Int, $filter: String, $publishers: [Uuid!]) {
        imprints(limit: $limit, offset: $offset, filter: $filter, publishers: $publishers) {
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
        imprintCount(filter: $filter, publishers: $publishers)
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
    pub order: Option<GenericOrderBy>,
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImprintsResponseData {
    pub imprints: Vec<Imprint>,
    pub imprint_count: i32,
}
