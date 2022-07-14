use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::work_relation::RelationType;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelationTypeDefinition {
    pub enum_values: Vec<RelationTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelationTypeValues {
    pub name: RelationType,
}

pub mod create_work_relation_mutation;
pub mod delete_work_relation_mutation;
pub mod relation_types_query;
pub mod update_work_relation_mutation;
