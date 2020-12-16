use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::ContributionType;

use super::Contribution;

const CREATE_CONTRIBUTION_MUTATION: &str = "
    mutation CreateContribution(
        $workId: Uuid!,
        $contributorId: Uuid!,
        $contributionType: ContributionType!,
        $mainContribution: Boolean!,
        $biography: String,
        $institution: String,
    ) {
        createContribution(data: {
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
            institution
            biography
            contributor {
                contributorId
                lastName
                fullName
            }
        }
    }
";

graphql_query_builder! {
    CreateContributionRequest,
    CreateContributionRequestBody,
    Variables,
    CREATE_CONTRIBUTION_MUTATION,
    CreateContributionResponseBody,
    CreateContributionResponseData,
    PushCreateContribution,
    PushActionCreateContribution
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
pub struct CreateContributionResponseData {
    pub create_contribution: Option<Contribution>,
}
