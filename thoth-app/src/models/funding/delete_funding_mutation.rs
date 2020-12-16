use serde::Deserialize;
use serde::Serialize;

const DELETE_FUNDING_MUTATION: &str = "
    mutation DeleteFunding(
        $fundingId: Uuid!
    ) {
        deleteFunding(
            fundingId: $fundingId
        ){
            fundingId
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
    pub funding_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlimFunding {
    pub funding_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFundingResponseData {
    pub delete_funding: Option<SlimFunding>,
}
