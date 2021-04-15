use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::Funder;

const UPDATE_FUNDER_MUTATION: &str = "
    mutation UpdateFunder(
        $funderId: Uuid!,
        $funderName: String!,
        $funderDoi: String
    ) {
        updateFunder(data: {
            funderId: $funderId
            funderName: $funderName
            funderDoi: $funderDoi
        }){
            funderId
            funderName
            updatedAt
        }
    }
";

graphql_query_builder! {
    UpdateFunderRequest,
    UpdateFunderRequestBody,
    Variables,
    UPDATE_FUNDER_MUTATION,
    UpdateFunderResponseBody,
    UpdateFunderResponseData,
    PushUpdateFunder,
    PushActionUpdateFunder
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFunderResponseData {
    pub update_funder: Option<Funder>,
}
