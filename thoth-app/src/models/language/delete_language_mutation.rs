use serde::Deserialize;
use serde::Serialize;
use thoth_api::language::model::LanguageCode;
use thoth_api::language::model::LanguageRelation;

use super::Language;

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
    pub language_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLanguageResponseData {
    pub delete_language: Option<Language>,
}
