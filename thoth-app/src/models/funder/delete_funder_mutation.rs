use serde::Deserialize;
use serde::Serialize;

use super::Funder;

const DELETE_FUNDER_MUTATION: &str = "
    mutation DeleteFunder(
        $funderId: Uuid!
    ) {
        deleteFunder(
            funderId: $funderId
        ){
            funderId
            funderName
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
    pub funder_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFunderResponseData {
    pub delete_funder: Option<Funder>,
}
