use serde::Deserialize;
use serde::Serialize;
use thoth_api::contribution::model::Contribution;
use thoth_api::contribution::model::ContributionType;
use uuid::Uuid;

const CREATE_CONTRIBUTION_MUTATION: &str = "
    mutation CreateContribution(
        $workId: Uuid!,
        $contributorId: Uuid!,
        $contributionType: ContributionType!,
        $mainContribution: Boolean!,
        $biography: String,
        $institution: String,
        $firstName: String,
        $lastName: String!,
        $fullName: String!,
        $contributionOrdinal: Int!,
    ) {
        createContribution(data: {
            workId: $workId
            contributorId: $contributorId
            contributionType: $contributionType
            mainContribution: $mainContribution
            biography: $biography
            institution: $institution
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
            biography
            institution
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
    pub work_id: Uuid,
    pub contributor_id: Uuid,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub contribution_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateContributionResponseData {
    pub create_contribution: Option<Contribution>,
}
