use serde::Deserialize;
use serde::Serialize;
use thoth_api::funder::model::Funder;
use thoth_api::model::Doi;
use uuid::Uuid;

const UPDATE_FUNDER_MUTATION: &str = "
    mutation UpdateFunder(
        $funderId: Uuid!,
        $funderName: String!,
        $funderDoi: Doi
    ) {
        updateFunder(data: {
            funderId: $funderId
            funderName: $funderName
            funderDoi: $funderDoi
        }){
            funderId
            funderName
            createdAt
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
    pub funder_doi: Option<Doi>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFunderResponseData {
    pub update_funder: Option<Funder>,
}
