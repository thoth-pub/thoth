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

query_builder! {
    FundersRequest,
    FundersRequestBody,
    FUNDERS_QUERY,
    FundersResponseBody,
    FundersResponseData,
    FetchFunders,
    FetchActionFunders
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FundersResponseData {
    pub funders: Vec<Funder>,
}

impl Default for FundersResponseData {
    fn default() -> FundersResponseData {
        FundersResponseData {
            funders: vec![],
        }
    }
}
