use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::location;
#[cfg(feature = "backend")]
use crate::schema::location_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    ExistingTypePath = "crate::schema::sql_types::LocationPlatform"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocationPlatform {
    #[cfg_attr(feature = "backend", db_rename = "Project MUSE")]
    #[strum(serialize = "Project MUSE")]
    ProjectMuse,
    #[cfg_attr(feature = "backend", db_rename = "OAPEN")]
    #[strum(serialize = "OAPEN")]
    Oapen,
    #[cfg_attr(feature = "backend", db_rename = "DOAB")]
    #[strum(serialize = "DOAB")]
    Doab,
    #[cfg_attr(feature = "backend", db_rename = "JSTOR")]
    #[strum(serialize = "JSTOR")]
    Jstor,
    #[cfg_attr(feature = "backend", db_rename = "EBSCO Host")]
    #[strum(serialize = "EBSCO Host")]
    EbscoHost,
    #[cfg_attr(feature = "backend", db_rename = "OCLC KB")]
    #[strum(serialize = "OCLC KB")]
    OclcKb,
    #[cfg_attr(feature = "backend", db_rename = "ProQuest KB")]
    #[strum(serialize = "ProQuest KB")]
    ProquestKb,
    #[cfg_attr(feature = "backend", db_rename = "ProQuest ExLibris")]
    #[strum(serialize = "ProQuest ExLibris")]
    ProquestExlibris,
    #[cfg_attr(feature = "backend", db_rename = "EBSCO KB")]
    #[strum(serialize = "EBSCO KB")]
    EbscoKb,
    #[cfg_attr(feature = "backend", db_rename = "JISC KB")]
    #[strum(serialize = "JISC KB")]
    JiscKb,
    #[cfg_attr(feature = "backend", db_rename = "Google Books")]
    #[strum(serialize = "Google Books")]
    GoogleBooks,
    #[cfg_attr(feature = "backend", db_rename = "Internet Archive")]
    #[strum(serialize = "Internet Archive")]
    InternetArchive,
    #[cfg_attr(feature = "backend", db_rename = "ScienceOpen")]
    #[strum(serialize = "ScienceOpen")]
    ScienceOpen,
    #[cfg_attr(feature = "backend", db_rename = "SciELO")]
    #[strum(serialize = "SciELO")]
    Scielo,
    #[cfg_attr(feature = "backend", db_rename = "Other")]
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
    pub account_id: Uuid,
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
    pub account_id: Uuid,
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
    assert_eq!(format!("{}", LocationPlatform::Scielo), "SciELO");
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
        LocationPlatform::from_str("SciELO").unwrap(),
        LocationPlatform::Scielo
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
