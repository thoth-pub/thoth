use serde::Deserialize;
use serde::Serialize;

use super::Contributor;

const DELETE_CONTRIBUTOR_MUTATION: &str = "
    mutation DeleteContributor(
        $contributorId: Uuid!
    ) {
        deleteContributor(
            contributorId: $contributorId
        ){
            contributorId
            lastName
            fullName
        }
    }
";

graphql_query_builder! {
    DeleteContributorRequest,
    DeleteContributorRequestBody,
    Variables,
    DELETE_CONTRIBUTOR_MUTATION,
    DeleteContributorResponseBody,
    DeleteContributorResponseData,
    PushDeleteContributor,
    PushActionDeleteContributor
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contributor_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteContributorResponseData {
    pub delete_contributor: Option<Contributor>,
}
