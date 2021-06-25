use serde::Deserialize;
use serde::Serialize;
use thoth_api::imprint::model::ImprintExtended;
use uuid::Uuid;

pub const IMPRINT_QUERY: &str = "
    query ImprintQuery($imprintId: Uuid!) {
        imprint(imprintId: $imprintId) {
            imprintId
            imprintName
            imprintUrl
            updatedAt
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
                createdAt
                updatedAt
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
    pub imprint_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ImprintResponseData {
    pub imprint: Option<ImprintExtended>,
}
