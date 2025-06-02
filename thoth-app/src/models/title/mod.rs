use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::language::LanguageCode;
use thoth_api::model::language::LanguageRelation;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCodeDefinition {
    pub enum_values: Vec<LanguageCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageCodeValues {
    pub name: LanguageCode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageRelationDefinition {
    pub enum_values: Vec<LanguageRelationValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguageRelationValues {
    pub name: LanguageRelation,
}

pub mod create_title_mutation;
pub mod update_title_mutation;


// pub mod delete_language_mutation;
// pub mod language_codes_query;
// pub mod language_relations_query;
