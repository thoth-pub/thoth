use serde::Deserialize;
use serde::Serialize;
use thoth_api::language::model::LanguageCode;
use thoth_api::language::model::LanguageRelation;

use super::Language;

const CREATE_LANGUAGE_MUTATION: &str = "
    mutation CreateLanguage(
        $workId: Uuid!,
        $languageCode: LanguageCode!,
        $languageRelation: LanguageRelation!,
        $mainLanguage: Boolean!
    ) {
        createLanguage(data: {
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
    CreateLanguageRequest,
    CreateLanguageRequestBody,
    Variables,
    CREATE_LANGUAGE_MUTATION,
    CreateLanguageResponseBody,
    CreateLanguageResponseData,
    PushCreateLanguage,
    PushActionCreateLanguage
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: String,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateLanguageResponseData {
    pub create_language: Option<Language>,
}
