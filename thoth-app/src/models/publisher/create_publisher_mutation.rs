use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::publisher::Publisher;

const CREATE_PUBLISHER_MUTATION: &str = "
    mutation CreatePublisher(
        $publisherName: String!,
        $publisherShortname: String
        $publisherUrl: String
    ) {
        createPublisher(data: {
            publisherName: $publisherName
            publisherShortname: $publisherShortname
            publisherUrl: $publisherUrl
        }){
            publisherId
            publisherName
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    CreatePublisherRequest,
    CreatePublisherRequestBody,
    Variables,
    CREATE_PUBLISHER_MUTATION,
    CreatePublisherResponseBody,
    CreatePublisherResponseData,
    PushCreatePublisher,
    PushActionCreatePublisher
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePublisherResponseData {
    pub create_publisher: Option<Publisher>,
}
