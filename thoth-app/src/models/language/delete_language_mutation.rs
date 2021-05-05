use serde::Deserialize;
use serde::Serialize;
use thoth_api::language::model::Language;
use uuid::Uuid;

const DELETE_LANGUAGE_MUTATION: &str = "
    mutation DeleteLanguage(
        $languageId: Uuid!
    ) {
        deleteLanguage(
            languageId: $languageId
        ){
            languageId
            workId
            languageCode
            languageRelation
            mainLanguage
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteLanguageRequest,
    DeleteLanguageRequestBody,
    Variables,
    DELETE_LANGUAGE_MUTATION,
    DeleteLanguageResponseBody,
    DeleteLanguageResponseData,
    PushDeleteLanguage,
    PushActionDeleteLanguage
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub language_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLanguageResponseData {
    pub delete_language: Option<Language>,
}
