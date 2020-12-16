use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const UPDATE_IMPRINT_MUTATION: &str = "
    mutation UpdateImprint(
        $imprintId: Uuid!,
        $imprintName: String!,
        $imprintUrl: String,
        $publisherId: Uuid!
    ) {
        updateImprint(data: {
            imprintId: $imprintId
            imprintName: $imprintName
            imprintUrl: $imprintUrl
            publisherId: $publisherId
        }){
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
    UpdateImprintRequest,
    UpdateImprintRequestBody,
    Variables,
    UPDATE_IMPRINT_MUTATION,
    UpdateImprintResponseBody,
    UpdateImprintResponseData,
    PushUpdateImprint,
    PushActionUpdateImprint
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub imprint_id: String,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateImprintResponseData {
    pub update_imprint: Option<Imprint>,
}
