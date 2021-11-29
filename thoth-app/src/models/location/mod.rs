use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::location::LocationPlatform;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LocationPlatformDefinition {
    pub enum_values: Vec<LocationPlatformValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LocationPlatformValues {
    pub name: LocationPlatform,
}

pub mod create_location_mutation;
pub mod delete_location_mutation;
pub mod location_platforms_query;
