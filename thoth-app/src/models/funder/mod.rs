use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Funder {
    pub funder_id: String,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

impl Default for Funder {
    fn default() -> Funder {
        Funder {
            funder_id: "".to_string(),
            funder_name: "".to_string(),
            funder_doi: None,
        }
    }
}

pub mod funders_query;
