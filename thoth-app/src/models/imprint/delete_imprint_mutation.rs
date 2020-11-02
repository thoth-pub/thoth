use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const DELETE_IMPRINT_MUTATION: &str = "
    mutation DeleteImprint(
        $imprintId: Uuid!
    ) {
        deleteImprint(
            imprintId: $imprintId
        ){
            imprintId
            imprintName
            imprintUrl
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
            }
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
    pub imprint_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteImprintResponseData {
    pub delete_imprint: Option<Imprint>,
}
