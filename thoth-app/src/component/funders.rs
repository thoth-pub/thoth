use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::funders_query::Variables;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::Funder;

pagination_component! {
    FundersComponent,
    Funder,
    funders,
    funder_count,
    FundersRequest,
    FetchActionFunders,
    FetchFunders,
    FundersRequestBody,
    Variables,
    SEARCH_FUNDERS,
    PAGINATION_COUNT_FUNDERS,
    vec!["ID".to_string(), "Funder".to_string(), "DOI".to_string()]
}
