use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::locale::LocaleCode;
use thoth_api::model::title::Title;
use uuid::Uuid;

const UPDATE_TITLE_MUTATION: &str = "
    mutation UpdateTitle(
        $titleId: Uuid!,
        $workId: Uuid!,
        $localeCode: LocaleCode!,
        $fullTitle: String!,
        $title: String!,
        $subtitle: String,
        $canonical: Boolean!
    ) {
        updateTitle(data: {
            titleId: $titleId,
            workId: $workId,
            localeCode: $localeCode,
            fullTitle: $fullTitle,
            title: $title,
            subtitle: $subtitle,
            canonical: $canonical
        }){
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
    UpdateTitleRequest,
    UpdateTitleRequestBody,
    Variables,
    UPDATE_TITLE_MUTATION,
    UpdateTitleResponseBody,
    UpdateTitleResponseData,
    PushUpdateTitle,
    PushActionUpdateTitle
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub title_id: Uuid,
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTitleResponseData {
    pub update_title: Option<Title>,
}
