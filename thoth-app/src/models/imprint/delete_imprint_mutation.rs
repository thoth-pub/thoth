use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::imprint::Imprint;
use uuid::Uuid;

const DELETE_IMPRINT_MUTATION: &str = "
    mutation DeleteImprint(
        $imprintId: Uuid!
    ) {
        deleteImprint(
            imprintId: $imprintId
        ){
            imprintId
            publisherId
            imprintName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteImprintRequest,
    DeleteImprintRequestBody,
    Variables,
    DELETE_IMPRINT_MUTATION,
    DeleteImprintResponseBody,
    DeleteImprintResponseData,
    PushDeleteImprint,
    PushActionDeleteImprint
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub imprint_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteImprintResponseData {
    pub delete_imprint: Option<Imprint>,
}
