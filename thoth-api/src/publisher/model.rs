use chrono::naive::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::publisher;
#[cfg(feature = "backend")]
use crate::schema::publisher_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting publishers list")
)]
pub enum PublisherField {
    PublisherID,
    PublisherName,
    PublisherShortname,
    PublisherURL,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Serialize, Deserialize)]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "publisher"
)]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "publisher"
)]
pub struct PatchPublisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct PublisherHistory {
    pub publisher_history_id: Uuid,
    pub publisher_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "publisher_history"
)]
pub struct NewPublisherHistory {
    pub publisher_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

impl fmt::Display for Publisher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.publisher_name)
    }
}
