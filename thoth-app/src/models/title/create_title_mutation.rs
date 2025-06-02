use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::title::Title;
use thoth_api::model::locale::LocaleCode;
use uuid::Uuid;

const CREATE_TITLE_MUTATION: &str = "
    mutation CreatTitle(
        $workId: Uuid!,
        $localeCode: LocaleCode!,
        $fullTitle: String!,
        $title: String!,
        $subtitle: String!,
        $canonical: Boolean!
    ) {
        createTitle(data: {
            workId: $Uuid!,
            localeCode: $LocaleCode!,
            fullTitle: $String!,
            title: $String!,
            subtitle: $String!,
            canonical: $Boolean!
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
    CreateTitleRequest,
    CreateTitleRequestBody,
    Variables,
    CREATE_TITLE_MUTATION,
    CreateTitleResponseBody,
    CreateTitleResponseData,
    PushCreateTitle,
    PushActionCreateTitle
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
    pub locale_code: LocaleCode,
    pub full_title: String,
    pub title: String,
    pub subtitle: String,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTitleResponseData {
    pub create_title: Option<Title>,
}
