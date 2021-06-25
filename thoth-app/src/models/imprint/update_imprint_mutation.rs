use serde::Deserialize;
use serde::Serialize;
use thoth_api::imprint::model::ImprintExtended;
use uuid::Uuid;

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
            updatedAt
            publisher {
                publisherId
                publisherName
                publisherShortname
                publisherUrl
                createdAt
                updatedAt
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
    pub imprint_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
    pub publisher_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateImprintResponseData {
    pub update_imprint: Option<ImprintExtended>,
}
