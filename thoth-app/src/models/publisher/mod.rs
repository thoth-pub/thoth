use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

impl Default for Publisher {
    fn default() -> Publisher {
        Publisher {
            publisher_id: "".to_string(),
            publisher_name: "".to_string(),
            publisher_shortname: None,
            publisher_url: None,
        }
    }
}

pub mod create_publisher_mutation;
pub mod publishers_query;
