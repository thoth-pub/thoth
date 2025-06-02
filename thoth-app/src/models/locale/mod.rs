use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::locale::LocaleCode;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocaleCodeDefinition {
    pub enum_values: Vec<LocaleCodeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocaleCodeValues {
    pub name: LocaleCode,
}

pub mod locale_codes_query;
