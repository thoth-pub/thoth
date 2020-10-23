use serde::Deserialize;
use serde::Serialize;

use super::funder::Funder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funding {
    pub funding_id: String,
    pub work_id: String,
    pub funder_id: String,
    pub program: Option<String>,
    pub project_name: Option<String>,
    pub project_shortname: Option<String>,
    pub grant_number: Option<String>,
    pub jurisdiction: Option<String>,
    pub funder: Funder,
}
