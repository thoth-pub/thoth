use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::contributor::Contributor;

pagination_component! {
    ContributorsComponent,
    Contributor,
    contributors,
    contributor_count,
    ContributorsRequest,
    FetchActionContributors,
    FetchContributors,
    ContributorsRequestBody,
    Variables,
    SEARCH_CONTRIBUTORS,
    PAGINATION_COUNT_CONTRIBUTORS,
    vec!["ID".to_string(), "FullName".to_string(), "ORCID".to_string()]
}
