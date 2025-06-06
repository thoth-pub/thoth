use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::title::Title;
use uuid::Uuid;

const DELETE_TITLE_MUTATION: &str = "
    mutation DeleteTitle(
        $titleId: Uuid!
    ) {
        deleteTitle(data: {
            titleId: $Uuid!
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
    DeleteTitleRequest,
    DeleteTitleRequestBody,
    Variables,
    DELETE_TITLE_MUTATION,
    DeleteTitleResponseBody,
    DeleteTitleResponseData,
    PushDeleteTitle,
    PushActionDeleteTitle
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub title_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTitleResponseData {
    pub delete_title: Option<Title>,
}
