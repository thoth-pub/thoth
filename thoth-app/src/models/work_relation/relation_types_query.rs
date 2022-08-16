use serde::Deserialize;
use serde::Serialize;

use super::RelationTypeDefinition;

const RELATION_TYPES_QUERY: &str = "
    {
        relation_types: __type(name: \"RelationType\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    RelationTypesRequest,
    RelationTypesRequestBody,
    Variables,
    RELATION_TYPES_QUERY,
    RelationTypesResponseBody,
    RelationTypesResponseData,
    FetchRelationTypes,
    FetchActionRelationTypes
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationTypesResponseData {
    pub relation_types: RelationTypeDefinition,
}
