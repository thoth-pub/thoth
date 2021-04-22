use serde::Deserialize;
use serde::Serialize;
use thoth_api::funder::model::Funder;
use uuid::Uuid;

pub const FUNDER_QUERY: &str = "
    query FunderQuery($funderId: Uuid!) {
        funder(funderId: $funderId) {
            funderId
            funderName
            funderDoi
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    FunderRequest,
    FunderRequestBody,
    Variables,
    FUNDER_QUERY,
    FunderResponseBody,
    FunderResponseData,
    FetchFunder,
    FetchActionFunder
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FunderResponseData {
    pub funder: Option<Funder>,
}
