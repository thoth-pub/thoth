use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::ContributionType;

use super::Contribution;

const UPDATE_CONTRIBUTION_MUTATION: &str = "
    mutation UpdateContribution(
        $workId: Uuid!,
        $contributorId: Uuid!,
        $contributionType: ContributionType!,
        $mainContribution: Boolean!,
        $biographqy: String,
        $institution: String,
    ) {
        updateContribution(data: {
            workId: $workId
            contributorId: $contributorId
            contributionType: $contributionType
            mainContribution: $mainContribution
            biography: $biography
            institution: $institution
        }){
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
    UpdateContributionRequest,
    UpdateContributionRequestBody,
    Variables,
    UPDATE_CONTRIBUTION_MUTATION,
    UpdateContributionResponseBody,
    UpdateContributionResponseData,
    PushUpdateContribution,
    PushActionUpdateContribution
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: String,
    pub contributor_id: String,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContributionResponseData {
    pub update_contribution: Option<Contribution>,
}
