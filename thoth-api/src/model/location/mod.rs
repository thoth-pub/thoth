use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::inputs::Direction;
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

#[test]
fn test_location_to_patch_location() {
    let location = Location {
        location_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap(),
        publication_id: Uuid::parse_str("00000000-0000-0000-AAAA-000000000002").unwrap(),
        landing_page: Some("https://www.book.com/pb_landing".to_string()),
        full_text_url: Some("https://example.com/full_text.pdf".to_string()),
        location_platform: LocationPlatform::PublisherWebsite,
        created_at: Default::default(),
        updated_at: Default::default(),
        canonical: true,
    };

    let patch_location = PatchLocation::from(location.clone());

    assert_eq!(patch_location.location_id, location.location_id);
    assert_eq!(patch_location.publication_id, location.publication_id);
    assert_eq!(patch_location.landing_page, location.landing_page);
    assert_eq!(patch_location.full_text_url, location.full_text_url);
    assert_eq!(patch_location.location_platform, location.location_platform);
    assert_eq!(patch_location.canonical, location.canonical);
}

#[test]
fn test_locationplatform_default() {
    let locationplatform: LocationPlatform = Default::default();
    assert_eq!(locationplatform, LocationPlatform::Other);
}

#[test]
fn test_locationplatform_display() {
    assert_eq!(format!("{}", LocationPlatform::ProjectMuse), "Project MUSE");
    assert_eq!(format!("{}", LocationPlatform::Oapen), "OAPEN");
    assert_eq!(format!("{}", LocationPlatform::Doab), "DOAB");
    assert_eq!(format!("{}", LocationPlatform::Jstor), "JSTOR");
    assert_eq!(format!("{}", LocationPlatform::EbscoHost), "EBSCO Host");
    assert_eq!(format!("{}", LocationPlatform::OclcKb), "OCLC KB");
    assert_eq!(format!("{}", LocationPlatform::ProquestKb), "ProQuest KB");
    assert_eq!(
        format!("{}", LocationPlatform::ProquestExlibris),
        "ProQuest ExLibris"
    );
    assert_eq!(format!("{}", LocationPlatform::EbscoKb), "EBSCO KB");
    assert_eq!(format!("{}", LocationPlatform::JiscKb), "JISC KB");
    assert_eq!(format!("{}", LocationPlatform::GoogleBooks), "Google Books");
    assert_eq!(
        format!("{}", LocationPlatform::InternetArchive),
        "Internet Archive"
    );
    assert_eq!(format!("{}", LocationPlatform::ScienceOpen), "ScienceOpen");
    assert_eq!(format!("{}", LocationPlatform::ScieloBooks), "SciELO Books");
    assert_eq!(format!("{}", LocationPlatform::Zenodo), "Zenodo");
    assert_eq!(
        format!("{}", LocationPlatform::PublisherWebsite),
        "Publisher Website"
    );
    assert_eq!(format!("{}", LocationPlatform::Thoth), "Thoth");
    assert_eq!(format!("{}", LocationPlatform::Other), "Other");
}

#[test]
fn test_locationplatform_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        LocationPlatform::from_str("Project MUSE").unwrap(),
        LocationPlatform::ProjectMuse
    );
    assert_eq!(
        LocationPlatform::from_str("OAPEN").unwrap(),
        LocationPlatform::Oapen
    );
    assert_eq!(
        LocationPlatform::from_str("DOAB").unwrap(),
        LocationPlatform::Doab
    );
    assert_eq!(
        LocationPlatform::from_str("JSTOR").unwrap(),
        LocationPlatform::Jstor
    );
    assert_eq!(
        LocationPlatform::from_str("EBSCO Host").unwrap(),
        LocationPlatform::EbscoHost
    );
    assert_eq!(
        LocationPlatform::from_str("OCLC KB").unwrap(),
        LocationPlatform::OclcKb
    );
    assert_eq!(
        LocationPlatform::from_str("ProQuest KB").unwrap(),
        LocationPlatform::ProquestKb
    );
    assert_eq!(
        LocationPlatform::from_str("ProQuest ExLibris").unwrap(),
        LocationPlatform::ProquestExlibris
    );
    assert_eq!(
        LocationPlatform::from_str("EBSCO KB").unwrap(),
        LocationPlatform::EbscoKb
    );
    assert_eq!(
        LocationPlatform::from_str("JISC KB").unwrap(),
        LocationPlatform::JiscKb
    );
    assert_eq!(
        LocationPlatform::from_str("Google Books").unwrap(),
        LocationPlatform::GoogleBooks
    );
    assert_eq!(
        LocationPlatform::from_str("Internet Archive").unwrap(),
        LocationPlatform::InternetArchive
    );
    assert_eq!(
        LocationPlatform::from_str("ScienceOpen").unwrap(),
        LocationPlatform::ScienceOpen
    );
    assert_eq!(
        LocationPlatform::from_str("SciELO Books").unwrap(),
        LocationPlatform::ScieloBooks
    );
    assert_eq!(
        LocationPlatform::from_str("Zenodo").unwrap(),
        LocationPlatform::Zenodo
    );
    assert_eq!(
        LocationPlatform::from_str("Publisher Website").unwrap(),
        LocationPlatform::PublisherWebsite
    );
    assert_eq!(
        LocationPlatform::from_str("Thoth").unwrap(),
        LocationPlatform::Thoth
    );
    assert_eq!(
        LocationPlatform::from_str("Other").unwrap(),
        LocationPlatform::Other
    );
    assert!(LocationPlatform::from_str("Amazon").is_err());
    assert!(LocationPlatform::from_str("Twitter").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::LocationPolicy;
