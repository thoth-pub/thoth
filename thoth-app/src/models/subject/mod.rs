use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::subject::SubjectType;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubjectTypeDefinition {
    pub enum_values: Vec<SubjectTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubjectTypeValues {
    pub name: SubjectType,
}

pub mod create_subject_mutation;
pub mod delete_subject_mutation;
pub mod subject_types_query;
