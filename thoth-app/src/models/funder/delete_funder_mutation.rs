use serde::Deserialize;
use serde::Serialize;
use thoth_api::funder::model::Funder;
use uuid::Uuid;

const DELETE_FUNDER_MUTATION: &str = "
    mutation DeleteFunder(
        $funderId: Uuid!
    ) {
        deleteFunder(
            funderId: $funderId
        ){
            funderId
            funderName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteFunderRequest,
    DeleteFunderRequestBody,
    Variables,
    DELETE_FUNDER_MUTATION,
    DeleteFunderResponseBody,
    DeleteFunderResponseData,
    PushDeleteFunder,
    PushActionDeleteFunder
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funder_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFunderResponseData {
    pub delete_funder: Option<Funder>,
}
