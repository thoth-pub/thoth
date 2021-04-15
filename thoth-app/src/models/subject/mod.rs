use serde::Deserialize;
use serde::Serialize;
use thoth_api::subject::model::SubjectType;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub subject_id: Uuid,
    pub subject_type: SubjectType,
    pub work_id: Uuid,
    pub subject_code: String,
    pub subject_ordinal: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubjectTypeDefinition {
    pub enum_values: Vec<SubjectTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubjectTypeValues {
    pub name: SubjectType,
}

impl Default for Subject {
    fn default() -> Subject {
        Subject {
            subject_id: Default::default(),
            subject_type: SubjectType::Keyword,
            work_id: Default::default(),
            subject_code: "".to_string(),
            subject_ordinal: 1,
        }
    }
}

pub mod create_subject_mutation;
pub mod delete_subject_mutation;
pub mod subject_types_query;
