use serde::Deserialize;
use serde::Serialize;

use super::publisher::Publisher;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Imprint {
    pub imprint_id: String,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher: Publisher,
}

impl Default for Imprint {
    fn default() -> Imprint {
        Imprint {
            imprint_id: "".to_string(),
            imprint_name: "".to_string(),
            imprint_url: None,
            publisher: Default::default(),
        }
    }
}

pub mod create_imprint_mutation;
pub mod imprints_query;
