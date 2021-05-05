use serde::Deserialize;
use serde::Serialize;
use thoth_api::contributor::model::Contributor;
use uuid::Uuid;

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
            createdAt
            updatedAt
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
    pub contributor_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteContributorResponseData {
    pub delete_contributor: Option<Contributor>,
}
