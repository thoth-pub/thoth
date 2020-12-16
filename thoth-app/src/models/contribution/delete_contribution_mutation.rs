use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::ContributionType;

use super::Contribution;

const DELETE_CONTRIBUTION_MUTATION: &str = "
    mutation DeleteContribution(
        $workId: Uuid!,
        $contributorId: Uuid!,
        $contributionType: ContributionType!
    ) {
        deleteContribution(
            workId: $workId
            contributorId: $contributorId
            contributionType: $contributionType
        ){
            workId
            contributorId
            contributionType
            mainContribution
            contributor {
                contributorId
                lastName
                fullName
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
    pub work_id: String,
    pub contributor_id: String,
    pub contribution_type: ContributionType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteContributionResponseData {
    pub delete_contribution: Option<Contribution>,
}
