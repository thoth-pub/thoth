use serde::Deserialize;
use serde::Serialize;
use thoth_api::language::model::LanguageCode;
use thoth_api::language::model::LanguageRelation;

use super::Language;

const UPDATE_LANGUAGE_MUTATION: &str = "
    mutation UpdateLanguage(
        $languageId: Uuid!,
        $workId: Uuid!,
        $languageCode: LanguageCode!,
        $languageRelation: LanguageRelation!,
        $mainLanguage: Boolean!
    ) {
        updateLanguage(data: {
            languageId: $languageId
            workId: $workId
            languageCode: $languageCode
            languageRelation: $languageRelation
            mainLanguage: $mainLanguage
        }){
            languageId
            workId
            languageCode
            languageRelation
            mainLanguage
        }
    }
";

graphql_query_builder! {
    UpdateLanguageRequest,
    UpdateLanguageRequestBody,
    Variables,
    UPDATE_LANGUAGE_MUTATION,
    UpdateLanguageResponseBody,
    UpdateLanguageResponseData,
    PushUpdateLanguage,
    PushActionUpdateLanguage
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub language_id: String,
    pub work_id: String,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLanguageResponseData {
    pub update_language: Option<Language>,
}
