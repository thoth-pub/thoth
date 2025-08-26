use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::contribution::Contribution;
use thoth_api::model::contribution::ContributionType;
use uuid::Uuid;

const UPDATE_CONTRIBUTION_MUTATION: &str = "
    mutation UpdateContribution(
        $contributionId: Uuid!,
        $workId: Uuid!,
        $contributorId: Uuid!,
        $contributionType: ContributionType!,
        $mainContribution: Boolean!,
        $firstName: String,
        $lastName: String!,
        $fullName: String!,
        $contributionOrdinal: Int!,
    ) {
        updateContribution(
            data: {
            contributionId: $contributionId
            workId: $workId
            contributorId: $contributorId
            contributionType: $contributionType
            mainContribution: $mainContribution
            firstName: $firstName
            lastName: $lastName
            fullName: $fullName
            contributionOrdinal: $contributionOrdinal
        }){
            contributionId
            workId
            contributorId
            contributionType
            mainContribution
            createdAt
            updatedAt
            firstName
            lastName
            fullName
            contributionOrdinal
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contribution_id: Uuid,
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContributionResponseData {
    pub update_contribution: Option<Contribution>,
}
