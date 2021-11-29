use serde::Deserialize;
use serde::Serialize;
use thoth_api::model::location::Location;
use thoth_api::model::location::LocationPlatform;
use uuid::Uuid;

const CREATE_LOCATION_MUTATION: &str = "
    mutation CreateLocation(
        $publicationId: Uuid!,
        $landingPage: String,
        $fullTextUrl: String,
        $locationPlatform: LocationPlatform!,
        $canonical: Boolean!
    ) {
        createLocation(data: {
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
            createdAt
            updatedAt
        }
    }
";

graphql_query_builder! {
    CreateLocationRequest,
    CreateLocationRequestBody,
    Variables,
    CREATE_LOCATION_MUTATION,
    CreateLocationResponseBody,
    CreateLocationResponseData,
    PushCreateLocation,
    PushActionCreateLocation
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Variables {
    pub publication_id: Uuid,
    pub landing_page: Option<String>,
    pub full_text_url: Option<String>,
    pub location_platform: LocationPlatform,
    pub canonical: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationResponseData {
    pub create_location: Option<Location>,
}
