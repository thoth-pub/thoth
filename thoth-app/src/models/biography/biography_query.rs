use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::biography::Biography;
use uuid::Uuid;

pub const BIOGRAPHY_QUERY: &str = "
    query BiographyQuery($biographyId: Uuid!, $markupFormat: MarkupFormat) {
        biography(biographyId: $biographyId, markupFormat: $markupFormat) {
            biographyId
            contributionId
            workId
            content
            canonical
            localeCode
        }
    }
";

graphql_query_builder! {
    BiographyRequest,
    BiographyRequestBody,
    Variables,
    BIOGRAPHY_QUERY,
    BiographyResponseBody,
    BiographyResponseData,
    FetchBiography,
    FetchActionBiography
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub biography_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BiographyResponseData {
    pub biography: Option<Biography>,
}
