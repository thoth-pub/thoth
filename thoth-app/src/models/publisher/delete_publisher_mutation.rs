use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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
            createdAt
            updatedAt
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
    pub publisher_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeletePublisherResponseData {
    pub delete_publisher: Option<Publisher>,
}
