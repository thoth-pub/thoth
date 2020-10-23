use serde::Deserialize;
use serde::Serialize;

use super::Funder;

pub const FUNDERS_QUERY: &str = "
    query FundersQuery($filter: String) {
        funders(limit: 9999, filter: $filter) {
            funderId
            funderName
            funderDoi
        }
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
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FundersResponseData {
    pub funders: Vec<Funder>,
}
