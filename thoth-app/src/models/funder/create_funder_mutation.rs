use serde::Deserialize;
use serde::Serialize;
use thoth_api::funder::model::Funder;

const CREATE_FUNDER_MUTATION: &str = "
    mutation CreateFunder(
        $funderName: String!,
        $funderDoi: String
    ) {
        createFunder(data: {
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
    CreateFunderRequest,
    CreateFunderRequestBody,
    Variables,
    CREATE_FUNDER_MUTATION,
    CreateFunderResponseBody,
    CreateFunderResponseData,
    PushCreateFunder,
    PushActionCreateFunder
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateFunderResponseData {
    pub create_funder: Option<Funder>,
}
