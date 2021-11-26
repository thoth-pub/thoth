use serde::Deserialize;
use serde::Serialize;

use super::LocationPlatformDefinition;

const LOCATION_PLATFORMS_QUERY: &str = "
    {
        location_platforms: __type(name: \"LocationPlatform\") {
            enumValues {
                name
            }
        }
    }
";

graphql_query_builder! {
    LocationPlatformsRequest,
    LocationPlatformsRequestBody,
    Variables,
    LOCATION_PLATFORMS_QUERY,
    LocationPlatformsResponseBody,
    LocationPlatformsResponseData,
    FetchLocationPlatforms,
    FetchActionLocationPlatforms
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Variables {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LocationPlatformsResponseData {
    pub location_platforms: LocationPlatformDefinition,
}
