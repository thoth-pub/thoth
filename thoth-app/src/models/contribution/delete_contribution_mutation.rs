use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::Contribution;
use uuid::Uuid;

const DELETE_CONTRIBUTION_MUTATION: &str = "
    mutation DeleteContribution(
        $contributionId: Uuid!,
    ) {
        deleteContribution(
            contributionId: $contributionId
        ){
            contributionId
            workId
            contributorId
            contributionType
            mainContribution
            createdAt
            updatedAt
            lastName
            fullName
        }
    }
";

graphql_query_builder! {
    DeleteContributionRequest,
    DeleteContributionRequestBody,
    Variables,
    DELETE_CONTRIBUTION_MUTATION,
    DeleteContributionResponseBody,
    DeleteContributionResponseData,
    PushDeleteContribution,
    PushActionDeleteContribution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteContributionResponseData {
    pub delete_contribution: Option<Contribution>,
}
