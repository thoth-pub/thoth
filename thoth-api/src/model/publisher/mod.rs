use serde::{Deserialize, Serialize};
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::publisher;
#[cfg(feature = "backend")]
use crate::schema::publisher_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting publishers list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublisherField {
    #[strum(serialize = "ID")]
    PublisherId,
    #[strum(serialize = "Name")]
    #[default]
    PublisherName,
    #[strum(serialize = "ShortName")]
    PublisherShortname,
    #[strum(serialize = "URL")]
    PublisherUrl,
    ZitadelId,
    AccessibilityStatement,
    AccessibilityReportUrl,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Subscription package determining which publisher services a publisher is entitled to"
    ),
    ExistingTypePath = "crate::schema::sql_types::ThothPackage"
)]
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ThothPackage {
    #[cfg_attr(
        feature = "backend",
        db_rename = "OASIS",
        graphql(description = "Default package, with no publisher-service capabilities")
    )]
    #[default]
    Oasis,
    #[cfg_attr(
        feature = "backend",
        db_rename = "OBELISK",
        graphql(
            description = "Package permitting OAI-PMH eligibility and private managed metrics collection"
        )
    )]
    Obelisk,
    #[cfg_attr(
        feature = "backend",
        db_rename = "SPHINX",
        graphql(
            description = "Package permitting OAI-PMH eligibility and all initial metrics capabilities"
        )
    )]
    Sphinx,
    #[cfg_attr(
        feature = "backend",
        db_rename = "PYRAMID",
        graphql(
            description = "Package permitting OAI-PMH eligibility and all initial metrics capabilities"
        )
    )]
    Pyramid,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(
        description = "Capability that a subscription package may grant to a publisher. A capability permits a feature but does not configure or activate it"
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PublisherCapability {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Publisher works may be considered for OAI-PMH after work-level open-licence and lifecycle checks"
        )
    )]
    OaiPmh,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth-managed drivers may collect and retain canonical metrics when a source account and platform/measure configuration are enabled"
        )
    )]
    MetricsCollect,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Publisher users may submit approved publisher-controlled usage or sales reports"
        )
    )]
    MetricsImport,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "A Thoth-owned authenticated service may serve publisher dashboard metrics"
        )
    )]
    MetricsDashboard,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "A Thoth-owned authenticated service may serve bounded work-level widget metrics"
        )
    )]
    MetricsWidget,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Eligible finalized canonical metrics may create and deliver OPERAS export claims"
        )
    )]
    MetricsOperasExport,
}

const OASIS_CAPABILITIES: &[PublisherCapability] = &[];

const OBELISK_CAPABILITIES: &[PublisherCapability] = &[
    PublisherCapability::OaiPmh,
    PublisherCapability::MetricsCollect,
];

const SPHINX_AND_PYRAMID_CAPABILITIES: &[PublisherCapability] = &[
    PublisherCapability::OaiPmh,
    PublisherCapability::MetricsCollect,
    PublisherCapability::MetricsImport,
    PublisherCapability::MetricsDashboard,
    PublisherCapability::MetricsWidget,
    PublisherCapability::MetricsOperasExport,
];

impl ThothPackage {
    /// The canonical package-to-capability mapping approved by ADR-0001 and
    /// `docs/engineering/decisions/package-capability-matrix.md`. Feature code
    /// must call `has_capability` rather than comparing package names.
    pub fn capabilities(self) -> &'static [PublisherCapability] {
        match self {
            ThothPackage::Oasis => OASIS_CAPABILITIES,
            ThothPackage::Obelisk => OBELISK_CAPABILITIES,
            ThothPackage::Sphinx => SPHINX_AND_PYRAMID_CAPABILITIES,
            ThothPackage::Pyramid => SPHINX_AND_PYRAMID_CAPABILITIES,
        }
    }

    pub fn has_capability(self, capability: PublisherCapability) -> bool {
        self.capabilities().contains(&capability)
    }
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub zitadel_id: Option<String>,
    pub accessibility_statement: Option<String>,
    pub accessibility_report_url: Option<String>,
    // Snapshots and API payloads serialized before BE-01 lack this field;
    // deserialization must not require it, so the documented OASIS default applies.
    #[serde(default)]
    pub subscription_package: ThothPackage,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new organisation that produces and distributes works"),
    diesel(table_name = publisher)
)]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub zitadel_id: Option<String>,
    pub accessibility_statement: Option<String>,
    pub accessibility_report_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing organisation that produces and distributes works"),
    diesel(table_name = publisher, treat_none_as_null = true)
)]
pub struct PatchPublisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub zitadel_id: Option<String>,
    pub accessibility_statement: Option<String>,
    pub accessibility_report_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct PublisherHistory {
    pub publisher_history_id: Uuid,
    pub publisher_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = publisher_history)
)]
pub struct NewPublisherHistory {
    pub publisher_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting publishers list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublisherOrderBy {
    pub field: PublisherField,
    pub direction: Direction,
}

impl fmt::Display for Publisher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.publisher_name)
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
pub mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::PublisherPolicy;
#[cfg(test)]
mod tests;
