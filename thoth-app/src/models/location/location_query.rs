use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::location::Location;
use uuid::Uuid;

pub const LOCATION_QUERY: &str = "
    query LocationQuery($locationId: Uuid!) {
        location(locationId: $locationId) {
            locationId
            publicationId
            landingPage
            fullTextUrl
            locationPlatform
            canonical
            createdAt
            updatedAt
            publication
            
        }
    }
";

graphql_query_builder! {
    LocationRequest,
    LocationRequestBody,
    Variables,
    LOCATION_QUERY,
    LocationResponseBody,
    LocationResponseData,
    FetchLocation,
    FetchActionLocation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub location_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LocationResponseData {
    pub publication: Option<Location>,
}
