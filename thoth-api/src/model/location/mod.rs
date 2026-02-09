use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::location;
#[cfg(feature = "backend")]
use crate::schema::location_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Platform where a publication is hosted or can be acquired"),
    ExistingTypePath = "crate::schema::sql_types::LocationPlatform"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocationPlatform {
    #[cfg_attr(
        feature = "backend",
        db_rename = "Project MUSE",
        graphql(description = "Project MUSE: https://muse.jhu.edu")
    )]
    #[strum(serialize = "Project MUSE")]
    ProjectMuse,
    #[cfg_attr(
        feature = "backend",
        db_rename = "OAPEN",
        graphql(
            description = "OAPEN (Open Access Publishing in European Networks): https://oapen.org"
        )
    )]
    #[strum(serialize = "OAPEN")]
    Oapen,
    #[cfg_attr(
        feature = "backend",
        db_rename = "DOAB",
        graphql(description = "DOAB (Directory of Open Access Books): https://doabooks.org")
    )]
    #[strum(serialize = "DOAB")]
    Doab,
    #[cfg_attr(
        feature = "backend",
        db_rename = "JSTOR",
        graphql(description = "JSTOR: https://jstor.org")
    )]
    #[strum(serialize = "JSTOR")]
    Jstor,
    #[cfg_attr(
        feature = "backend",
        db_rename = "EBSCO Host",
        graphql(description = "EBSCO Host")
    )]
    #[strum(serialize = "EBSCO Host")]
    EbscoHost,
    #[cfg_attr(
        feature = "backend",
        db_rename = "OCLC KB",
        graphql(description = "OCLC Knowledge Base")
    )]
    #[strum(serialize = "OCLC KB")]
    OclcKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ProQuest KB",
        graphql(description = "ProQuest Knowledge Base")
    )]
    #[strum(serialize = "ProQuest KB")]
    ProquestKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ProQuest ExLibris",
        graphql(description = "ProQuest ExLibris")
    )]
    #[strum(serialize = "ProQuest ExLibris")]
    ProquestExlibris,
    #[cfg_attr(
        feature = "backend",
        db_rename = "EBSCO KB",
        graphql(description = "EBSCO Knowledge Base")
    )]
    #[strum(serialize = "EBSCO KB")]
    EbscoKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "JISC KB",
        graphql(description = "JISC Knowledge Base")
    )]
    #[strum(serialize = "JISC KB")]
    JiscKb,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Google Books",
        graphql(description = "Google Books: https://books.google.com")
    )]
    #[strum(serialize = "Google Books")]
    GoogleBooks,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Internet Archive",
        graphql(description = "Internet Archive: https://archive.org")
    )]
    #[strum(serialize = "Internet Archive")]
    InternetArchive,
    #[cfg_attr(
        feature = "backend",
        db_rename = "ScienceOpen",
        graphql(description = "ScienceOpen: https://scienceopen.com")
    )]
    #[strum(serialize = "ScienceOpen")]
    ScienceOpen,
    #[cfg_attr(
        feature = "backend",
        db_rename = "SciELO Books",
        graphql(
            description = "SciELO (Scientific Electronic Library Online) Books: https://books.scielo.org"
        )
    )]
    #[strum(serialize = "SciELO Books")]
    ScieloBooks,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Zenodo",
        graphql(description = "Zenodo: https://zenodo.org")
    )]
    #[strum(serialize = "Zenodo")]
    Zenodo,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Publisher Website",
        graphql(description = "Publisher's own website")
    )]
    #[strum(serialize = "Publisher Website")]
    PublisherWebsite,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Thoth",
        graphql(description = "Publisher CDN hosted by Thoth")
    )]
    #[strum(serialize = "Thoth")]
    Thoth,
    #[cfg_attr(
        feature = "backend",
        db_rename = "Other",
        graphql(description = "Another platform not listed above")
    )]
    #[default]
    Other,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting locations list")
)]
pub enum LocationField {
    LocationId,
    PublicationId,
    LandingPage,
    FullTextUrl,
    LocationPlatform,
    Canonical,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub location_id: Uuid,
    pub publication_id: Uuid,
    pub landing_page: Option<String>,
    pub full_text_url: Option<String>,
    pub location_platform: LocationPlatform,
    pub canonical: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new location (such as a web shop or distribution platform) where a publication can be acquired or viewed"),
    diesel(table_name = location)
)]
pub struct NewLocation {
    pub publication_id: Uuid,
    pub landing_page: Option<String>,
    pub full_text_url: Option<String>,
    pub location_platform: LocationPlatform,
    pub canonical: bool,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing location (such as a web shop or distribution platform) where a publication can be acquired or viewed"),
    diesel(table_name = location, treat_none_as_null = true)
)]
pub struct PatchLocation {
    pub location_id: Uuid,
    pub publication_id: Uuid,
    pub landing_page: Option<String>,
    pub full_text_url: Option<String>,
    pub location_platform: LocationPlatform,
    pub canonical: bool,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct LocationHistory {
    pub location_history_id: Uuid,
    pub location_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = location_history)
)]
pub struct NewLocationHistory {
    pub location_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting locations list")
)]
pub struct LocationOrderBy {
    pub field: LocationField,
    pub direction: Direction,
}

impl Default for LocationOrderBy {
    fn default() -> LocationOrderBy {
        LocationOrderBy {
            field: LocationField::LocationPlatform,
            direction: Default::default(),
        }
    }
}

impl From<Location> for PatchLocation {
    fn from(location: Location) -> Self {
        PatchLocation {
            location_id: location.location_id,
            publication_id: location.publication_id,
            landing_page: location.landing_page,
            full_text_url: location.full_text_url,
            location_platform: location.location_platform,
            canonical: location.canonical,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::LocationPolicy;
#[cfg(test)]
mod tests;
