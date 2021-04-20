use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::Contributor;

const UPDATE_CONTRIBUTOR_MUTATION: &str = "
    mutation UpdateContributor(
        $contributorId: Uuid!,
        $firstName: String,
        $lastName: String!,
        $fullName: String!,
        $orcid: String,
        $website: String
    ) {
        updateContributor(data: {
            contributorId: $contributorId
            firstName: $firstName
            lastName: $lastName
            fullName: $fullName
            orcid: $orcid
            website: $website
        }){
            contributorId
            lastName
            fullName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    UpdateContributorRequest,
    UpdateContributorRequestBody,
    Variables,
    UPDATE_CONTRIBUTOR_MUTATION,
    UpdateContributorResponseBody,
    UpdateContributorResponseData,
    PushUpdateContributor,
    PushActionUpdateContributor
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContributorResponseData {
    pub update_contributor: Option<Contributor>,
}
