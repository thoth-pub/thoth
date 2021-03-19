use serde::Deserialize;
use serde::Serialize;
use thoth_api::funder::model::FunderOrderBy;

use super::Funder;

pub const FUNDERS_QUERY: &str = "
    query FundersQuery($limit: Int, $offset: Int, $filter: String) {
        funders(limit: $limit, offset: $offset, filter: $filter) {
            funderId
            funderName
            funderDoi
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
