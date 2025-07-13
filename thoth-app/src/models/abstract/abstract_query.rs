use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::r#abstract::Abstract;
use uuid::Uuid;

pub const ABSTRACT_QUERY: &str = "
    query AbstractQuery($abstractId: Uuid!) {
        abstract(abstractId: $abstractId) {
            abstractId
            workId
            content
            localeCode
            abstractType
            canonical
        }
    }
";

graphql_query_builder! {
    AbstractRequest,
    AbstractRequestBody,
    Variables,
    ABSTRACT_QUERY,
    AbstractResponseBody,
    AbstractResponseData,
    FetchAbstract,
    FetchActionAbstract
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub abstract_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AbstractResponseData {
    pub r#abstract: Option<Abstract>,
} 