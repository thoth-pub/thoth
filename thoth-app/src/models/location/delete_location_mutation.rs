use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::location::Location;
use uuid::Uuid;

const DELETE_LOCATION_MUTATION: &str = "
    mutation DeleteLocation(
        $locationId: Uuid!
    ) {
        deleteLocation(
            locationId: $locationId
        ){
            locationId
            publicationId
            landingPage
            locationPlatform
            canonical
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    DeleteLocationRequest,
    DeleteLocationRequestBody,
    Variables,
    DELETE_LOCATION_MUTATION,
    DeleteLocationResponseBody,
    DeleteLocationResponseData,
    PushDeleteLocation,
    PushActionDeleteLocation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub location_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLocationResponseData {
    pub delete_location: Option<Location>,
}
