use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::funder::Funder;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funding {
    pub funding_id: Uuid,
    pub work_id: Uuid,
    pub funder_id: Uuid,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
    pub funder: Funder,
}

pub mod create_funding_mutation;
pub mod delete_funding_mutation;
