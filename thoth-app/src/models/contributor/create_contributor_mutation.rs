use serde::Deserialize;
use serde::Serialize;

use super::Contributor;

const CREATE_CONTRIBUTOR_MUTATION: &str = "
    mutation CreateContributor(
        $firstName: String,
        $lastName: String!,
        $fullName: String!,
        $orcid: String,
        $website: String
    ) {
        createContributor(data: {
            firstName: $firstName
            lastName: $lastName
            fullName: $fullName
            orcid: $orcid
            website: $website
        }){
            contributorId
            lastName
            fullName
            updatedAt
        }
    }
";

graphql_query_builder! {
    CreateContributorRequest,
    CreateContributorRequestBody,
    Variables,
    CREATE_CONTRIBUTOR_MUTATION,
    CreateContributorResponseBody,
    CreateContributorResponseData,
    PushCreateContributor,
    PushActionCreateContributor
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateContributorResponseData {
    pub create_contributor: Option<Contributor>,
}
