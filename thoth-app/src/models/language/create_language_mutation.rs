use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::language::Language;
use thoth_api::model::language::LanguageCode;
use thoth_api::model::language::LanguageRelation;
use uuid::Uuid;

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
            createdAt
            updatedAt
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub work_id: Uuid,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateLanguageResponseData {
    pub create_language: Option<Language>,
}
