use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::locale::LocaleCode;
use thoth_api::model::r#abstract::Abstract;
use thoth_api::model::r#abstract::AbstractOrderBy;
use thoth_api::model::MarkupFormat;
use uuid::Uuid;

pub const ABSTRACTS_QUERY: &str = "
    query AbstractsQuery(
        $limit: Int,
        $offset: Int,
        $filter: String,
        $order: AbstractOrderBy,
        $localeCodes: [LocaleCode!],
        $markupFormat: MarkupFormat!
    ) {
        abstracts(
            limit: $limit,
            offset: $offset,
            filter: $filter,
            order: $order,
            localeCodes: $localeCodes,
            markupFormat: $markupFormat
        ) {
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
    AbstractsRequest,
    AbstractsRequestBody,
    Variables,
    ABSTRACTS_QUERY,
    AbstractsResponseBody,
    AbstractsResponseData,
    FetchAbstracts,
    FetchActionAbstracts
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub filter: Option<String>,
    pub order: Option<AbstractOrderBy>,
    pub locale_codes: Option<Vec<LocaleCode>>,
    pub markup_format: MarkupFormat,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AbstractsResponseData {
    pub abstracts: Vec<Abstract>,
} 