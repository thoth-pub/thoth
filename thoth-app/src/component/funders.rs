use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::funders_query::Variables;
use thoth_api::model::funder::Funder;
use thoth_api::model::funder::FunderField;
use thoth_api::model::funder::FunderOrderBy;

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
