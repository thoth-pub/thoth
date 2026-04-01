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
