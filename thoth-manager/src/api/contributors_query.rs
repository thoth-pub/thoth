use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Contributor;

const CONTRIBUTORS_QUERY: &str = "
    {
        contributors(limit: 9999) {
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
    ContributorsRequest,
    ContributorsRequestBody,
    CONTRIBUTORS_QUERY,
    ContributorsResponseBody,
    ContributorsResponseData,
    FetchContributors,
    FetchActionContributors
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContributorsResponseData {
    pub contributors: Vec<Contributor>,
}

impl Default for ContributorsResponseData {
    fn default() -> ContributorsResponseData {
        ContributorsResponseData {
            contributors: vec![],
        }
    }
}
