use serde::Deserialize;
use serde::Serialize;

use super::super::GraphqlFieldList;

const REFERENCE_FIELDS_QUERY: &str = "
    {
        reference_fields: __type(name: \"Reference\") {
            fields {
                name
                description
            }
        }
    }
";

graphql_query_builder! {
    ReferenceFieldsRequest,
    ReferenceFieldsRequestBody,
    Variables,
    REFERENCE_FIELDS_QUERY,
    ReferenceFieldsResponseBody,
    ReferenceFieldsResponseData,
    FetchReferenceFields,
    FetchActionReferenceFields
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceFieldsResponseData {
    pub reference_fields: GraphqlFieldList,
}
