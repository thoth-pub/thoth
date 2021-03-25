use serde::Deserialize;
use serde::Serialize;

use super::Imprint;

const CREATE_IMPRINT_MUTATION: &str = "
    mutation CreateImprint(
            $imprintName: String!,
            $imprintUrl: String,
            $publisherId: Uuid!
    ) {
        createImprint(data: {
            imprintName: $imprintName
            imprintUrl: $imprintUrl
            publisherId: $publisherId
        }){
            imprintId
            imprintName
            imprintUrl
            updatedAt
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
    CreateImprintRequest,
    CreateImprintRequestBody,
    Variables,
    CREATE_IMPRINT_MUTATION,
    CreateImprintResponseBody,
    CreateImprintResponseData,
    PushCreateImprint,
    PushActionCreateImprint
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateImprintResponseData {
    pub create_imprint: Option<Imprint>,
}
