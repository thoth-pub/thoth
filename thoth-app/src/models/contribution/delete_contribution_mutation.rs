use serde::Deserialize;
use serde::Serialize;

use super::Contribution;

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
            lastName
            fullName
            contributor {
                contributorId
                lastName
                fullName
                updatedAt
            }
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
    pub contribution_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteContributionResponseData {
    pub delete_contribution: Option<Contribution>,
}
