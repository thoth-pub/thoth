use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::funder::Funder;
use thoth_api::model::funder::FunderOrderBy;

pub const FUNDERS_QUERY: &str = "
    query FundersQuery($limit: Int, $offset: Int, $filter: String, $order: FunderOrderBy) {
        funders(limit: $limit, offset: $offset, filter: $filter, order: $order) {
            funderId
            funderName
            funderDoi
            createdAt
            updatedAt
        }
        funderCount(filter: $filter)
    }
";

graphql_query_builder! {
    FundersRequest,
    FundersRequestBody,
    Variables,
    FUNDERS_QUERY,
    FundersResponseBody,
    FundersResponseData,
    FetchFunders,
    FetchActionFunders
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<FunderOrderBy>,
    // Unused, but required by pagination_component macro
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FundersResponseData {
    pub funders: Vec<Funder>,
    pub funder_count: i32,
}
