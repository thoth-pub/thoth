use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::institution::Institution;
use thoth_api::model::institution::InstitutionOrderBy;

pub const INSTITUTIONS_QUERY: &str = "
    query InstitutionsQuery($limit: Int, $offset: Int, $filter: String, $order: InstitutionOrderBy) {
        institutions(limit: $limit, offset: $offset, filter: $filter, order: $order) {
            institutionId
            institutionName
            institutionDoi
            ror
            countryCode
            createdAt
            updatedAt
        }
        institutionCount(filter: $filter)
    }
";

graphql_query_builder! {
    InstitutionsRequest,
    InstitutionsRequestBody,
    Variables,
    INSTITUTIONS_QUERY,
    InstitutionsResponseBody,
    InstitutionsResponseData,
    FetchInstitutions,
    FetchActionInstitutions
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<InstitutionOrderBy>,
    // Unused, but required by pagination_component macro
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionsResponseData {
    pub institutions: Vec<Institution>,
    pub institution_count: i32,
}
