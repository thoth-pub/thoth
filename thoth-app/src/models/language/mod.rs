use serde::Deserialize;
use serde::Serialize;
use thoth_api::language::model::LanguageCode;
use thoth_api::language::model::LanguageRelation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub language_id: String,
    pub work_id: String,
    pub language_code: LanguageCode,
    pub language_relation: LanguageRelation,
    pub main_language: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCodeDefinition {
    pub enum_values: Vec<LanguageCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCodeValues {
    pub name: LanguageCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageRelationDefinition {
    pub enum_values: Vec<LanguageRelationValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageRelationValues {
    pub name: LanguageRelation,
}

impl Default for Language {
    fn default() -> Language {
        Language {
            language_id: "".to_string(),
            work_id: "".to_string(),
            language_code: LanguageCode::Eng,
            language_relation: LanguageRelation::Original,
            main_language: true,
        }
    }
}

pub mod language_codes_query;
pub mod language_relations_query;
