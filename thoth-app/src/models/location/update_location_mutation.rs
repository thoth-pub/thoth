use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::location::Location;
use thoth_api::model::location::LocationPlatform;
use uuid::Uuid;

const UPDATE_LOCATION_MUTATION: &str = "
    mutation UpdateLocation(
        $locationId: Uuid!,
        $publicationId: Uuid!,
        $landingPage: String,
        $fullTextUrl: String,
        $locationPlatform: LocationPlatform!,
        $canonical: Boolean!
    ) {
        updateLocation(data: {
            locationId: $locationId
            publicationId: $publicationId
            landingPage: $landingPage
            fullTextUrl: $fullTextUrl
            locationPlatform: $locationPlatform
            canonical: $canonical
        }){
            locationId
            publicationId
            landingPage
            fullTextUrl
            locationPlatform
            canonical
        }
    }
";

graphql_query_builder! {
    UpdateLocationRequest,
    UpdateLocationRequestBody,
    Variables,
    UPDATE_LOCATION_MUTATION,
    UpdateLocationResponseBody,
    UpdateLocationResponseData,
    PushUpdateLocation,
    PushActionUpdateLocation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub location_id: Uuid,
    pub publication_id: Uuid,
    pub landing_page: Option<String>,
    pub full_text_url: Option<String>,
    pub location_platform: LocationPlatform,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationResponseData {
    pub update_location: Option<Location>,
}
