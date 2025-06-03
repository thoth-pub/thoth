use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::Title;
use uuid::Uuid;

pub const TITLE_QUERY: &str = "
    query TitleQuery($titleId: Uuid!) {
        title(titleId: $titleId) {
            titleId
            workId
            localeCode
            fullTitle
            title
            subtitle
            canonical
        }
    }
";

graphql_query_builder! {
    TitleRequest,
    TitleRequestBody,
    Variables,
    TITLE_QUERY,
    TitleResponseBody,
    TitleResponseData,
    FetchTitle,
    FetchActionTitle
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub title_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TitleResponseData {
    pub title: Option<Title>,
}
