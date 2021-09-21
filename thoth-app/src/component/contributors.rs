use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use thoth_api::model::contributor::Contributor;
use thoth_api::model::contributor::ContributorField;
use thoth_api::model::contributor::ContributorOrderBy;

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
    vec![
        ContributorField::ContributorId.to_string(),
        ContributorField::FullName.to_string(),
        ContributorField::Orcid.to_string(),
        ContributorField::UpdatedAt.to_string(),
    ],
    ContributorOrderBy,
    ContributorField,
}
