use serde::Deserialize;
use serde::Serialize;

use super::Contributor;

pub const CONTRIBUTORS_QUERY: &str = "
    query ContributorsQuery($filter: String) {
        contributors(limit: 9999, filter: $filter) {
            contributorId
            firstName
            lastName
            fullName
            orcid
            website
        }
    }
";

graphql_query_builder! {
    ContributorsRequest,
    ContributorsRequestBody,
    Variables,
    CONTRIBUTORS_QUERY,
    ContributorsResponseBody,
    ContributorsResponseData,
    FetchContributors,
    FetchActionContributors
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContributorsResponseData {
    pub contributors: Vec<Contributor>,
}
