use serde::Deserialize;
use serde::Serialize;

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
        }
    }
";

query_builder! {
    ContributorRequest,
    ContributorRequestBody,
    CONTRIBUTOR_QUERY,
    ContributorResponseBody,
    ContributorResponseData,
    FetchContributor,
    FetchActionContributor
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributorResponseData {
    pub contributor: Option<Contributor>,
}

impl Default for ContributorResponseData {
    fn default() -> ContributorResponseData {
        ContributorResponseData { contributor: None }
    }
}
