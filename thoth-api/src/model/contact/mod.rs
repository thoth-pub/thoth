use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::contact;
#[cfg(feature = "backend")]
use crate::schema::contact_history;

#[cfg_attr(
    feature = "backend",
    derive(DbEnum, juniper::GraphQLEnum),
    graphql(description = "Type of a contact"),
    ExistingTypePath = "crate::schema::sql_types::ContactType"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactType {
    #[cfg_attr(
        feature = "backend",
        db_rename = "Accessibility",
        graphql(description = "Contact for accessibility queries")
    )]
    #[default]
    Accessibility,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting contacts list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactField {
    ContactId,
    PublisherId,
    ContactType,
    #[default]
    Email,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub contact_id: Uuid,
    pub publisher_id: Uuid,
    pub contact_type: ContactType,
    pub email: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    graphql(description = "Set of values required to define a new way of getting in touch with a publisher"),
    diesel(table_name = contact)
)]
pub struct NewContact {
    pub publisher_id: Uuid,
    pub contact_type: ContactType,
    pub email: String,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    graphql(description = "Set of values required to update an existing way of getting in touch with a publisher"),
    diesel(table_name = contact, treat_none_as_null = true)
)]
pub struct PatchContact {
    pub contact_id: Uuid,
    pub publisher_id: Uuid,
    pub contact_type: ContactType,
    pub email: String,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct ContactHistory {
    pub contact_history_id: Uuid,
    pub contact_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    diesel(table_name = contact_history)
)]
pub struct NewContactHistory {
    pub contact_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting contacts list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactOrderBy {
    pub field: ContactField,
    pub direction: Direction,
}

#[test]
fn test_contactfield_default() {
    let contfield: ContactField = Default::default();
    assert_eq!(contfield, ContactField::Email);
}

#[cfg(feature = "backend")]
pub mod crud;
