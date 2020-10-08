use serde::Deserialize;
use serde::Serialize;
use thoth_api::models::contributor::ContributionType;

use crate::api::models::contributor::Contributor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub work_id: String,
    pub contributor_id: String,
    pub contribution_type: ContributionType,
    pub main_contribution: bool,
    pub biography: Option<String>,
    pub institution: Option<String>,
    pub contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeDefinition {
    pub enum_values: Vec<ContributionTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionTypeValues {
    pub name: ContributionType,
}
