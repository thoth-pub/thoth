use serde::Deserialize;
use serde::Serialize;
use thoth_api::publication::model::PublicationType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub publication_id: String,
    pub publication_type: PublicationType,
    pub work_id: String,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeDefinition {
    pub enum_values: Vec<PublicationTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicationTypeValues {
    pub name: PublicationType,
}

impl Default for Publication {
    fn default() -> Publication {
        Publication {
            publication_id: "".to_string(),
            publication_type: PublicationType::Paperback,
            work_id: "".to_string(),
            isbn: None,
            publication_url: None,
        }
    }
}

pub mod create_publication_mutation;
pub mod delete_publication_mutation;
pub mod publication_types_query;
pub mod publication_query;
pub mod publications_query;
pub mod update_publication_mutation;
