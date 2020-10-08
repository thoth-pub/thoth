use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub contributor_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

impl Default for Contributor {
    fn default() -> Contributor {
        Contributor {
            contributor_id: "".to_string(),
            first_name: None,
            last_name: None,
            full_name: "".to_string(),
            orcid: None,
            website: None,
        }
    }
}
