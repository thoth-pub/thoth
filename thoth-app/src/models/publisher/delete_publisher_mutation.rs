use serde::Deserialize;
use serde::Serialize;

use super::Publisher;

const DELETE_PUBLISHER_MUTATION: &str = "
    mutation DeletePublisher(
        $publisherId: Uuid!
    ) {
        deletePublisher(
            publisherId: $publisherId
        ){
            publisherId
            publisherName
        }
    }
";

graphql_query_builder! {
    DeletePublisherRequest,
    DeletePublisherRequestBody,
    Variables,
    DELETE_PUBLISHER_MUTATION,
    DeletePublisherResponseBody,
    DeletePublisherResponseData,
    PushDeletePublisher,
    PushActionDeletePublisher
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publisher_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePublisherResponseData {
    pub delete_publisher: Option<Publisher>,
}
