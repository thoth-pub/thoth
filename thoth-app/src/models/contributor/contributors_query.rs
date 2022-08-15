use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::contributor::Contributor;
use thoth_api::model::contributor::ContributorOrderBy;

pub const CONTRIBUTORS_QUERY: &str = "
    query ContributorsQuery($limit: Int, $offset: Int, $filter: String, $order: ContributorOrderBy) {
        contributors(limit: $limit, offset: $offset, filter: $filter, order: $order) {
            contributorId
            firstName
            lastName
            fullName
            orcid
            website
            createdAt
            updatedAt
        }
        contributorCount(filter: $filter)
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<ContributorOrderBy>,
    // Unused, but required by pagination_component macro
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContributorsResponseData {
    pub contributors: Vec<Contributor>,
    pub contributor_count: i32,
}
