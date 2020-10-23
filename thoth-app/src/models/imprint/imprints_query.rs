use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const IMPRINTS_QUERY: &str = "
    {
        imprints(limit: 9999) {
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
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImprintsResponseData {
    pub imprints: Vec<Imprint>,
}
