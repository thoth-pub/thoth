use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

pub const IMPRINT_QUERY: &str = "
    query ImprintQuery($imprintId: Uuid!) {
        imprint(imprintId: $imprintId) {
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
    ImprintRequest,
    ImprintRequestBody,
    Variables,
    IMPRINT_QUERY,
    ImprintResponseBody,
    ImprintResponseData,
    FetchImprint,
    FetchActionImprint
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub imprint_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImprintResponseData {
    pub imprint: Option<Imprint>,
}
