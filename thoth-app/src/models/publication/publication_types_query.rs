use serde::Deserialize;
use serde::Serialize;

use super::PublicationTypeDefinition;

const PUBLICATION_TYPES_QUERY: &str = "
    {
        publication_types: __type(name: \"PublicationType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    PublicationTypesRequest,
    PublicationTypesRequestBody,
    PUBLICATION_TYPES_QUERY,
    PublicationTypesResponseBody,
    PublicationTypesResponseData,
    FetchPublicationTypes,
    FetchActionPublicationTypes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicationTypesResponseData {
    pub publication_types: PublicationTypeDefinition,
}

impl Default for PublicationTypesResponseData {
    fn default() -> PublicationTypesResponseData {
        PublicationTypesResponseData {
            publication_types: PublicationTypeDefinition {
                enum_values: vec![],
            },
        }
    }
}
