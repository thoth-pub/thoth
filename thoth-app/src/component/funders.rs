use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::funders_query::Variables;
use crate::models::funder::Funder;
use thoth_api::funder::model::FunderField;
use thoth_api::funder::model::FunderOrderBy;

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
    vec![
        FunderField::FunderId.to_string(),
        FunderField::FunderName.to_string(),
        FunderField::FunderDoi.to_string(),
        FunderField::UpdatedAt.to_string(),
    ],
    FunderOrderBy,
    FunderField,
}
