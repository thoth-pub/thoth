use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::Contributor;

pub const CONTRIBUTOR_QUERY: &str = "
    query ContributorQuery($contributorId: Uuid!) {
        contributor(contributorId: $contributorId) {
            contributorId
            firstName
            lastName
            fullName
            orcid
            website
            updatedAt
        }
    }
";

graphql_query_builder! {
    ContributorRequest,
    ContributorRequestBody,
    Variables,
    CONTRIBUTOR_QUERY,
    ContributorResponseBody,
    ContributorResponseData,
    FetchContributor,
    FetchActionContributor
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub contributor_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorResponseData {
    pub contributor: Option<Contributor>,
}
