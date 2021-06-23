use serde::Deserialize;
use serde::Serialize;
use thoth_api::funding::model::Funding;
use uuid::Uuid;

const DELETE_FUNDING_MUTATION: &str = "
    mutation DeleteFunding(
        $fundingId: Uuid!
    ) {
        deleteFunding(
            fundingId: $fundingId
        ){
            fundingId
            workId
            funderId
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteFundingRequest,
    DeleteFundingRequestBody,
    Variables,
    DELETE_FUNDING_MUTATION,
    DeleteFundingResponseBody,
    DeleteFundingResponseData,
    PushDeleteFunding,
    PushActionDeleteFunding
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funding_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFundingResponseData {
    pub delete_funding: Option<Funding>,
}
