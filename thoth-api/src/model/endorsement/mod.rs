use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::endorsement;
#[cfg(feature = "backend")]
use crate::schema::endorsement_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting endorsements list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndorsementField {
    EndorsementId,
    WorkId,
    #[default]
    EndorsementOrdinal,
    AuthorName,
    AuthorRole,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Endorsement {
    pub endorsement_id: Uuid,
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new endorsement linked to a work"),
    diesel(table_name = endorsement)
)]
pub struct NewEndorsement {
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::AsChangeset),
    graphql(description = "Set of values required to update an existing endorsement"),
    diesel(table_name = endorsement, treat_none_as_null = true)
)]
pub struct PatchEndorsement {
    pub endorsement_id: Uuid,
    pub work_id: Uuid,
    pub author_name: Option<String>,
    pub author_role: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub endorsement_ordinal: i32,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct EndorsementHistory {
    pub endorsement_history_id: Uuid,
    pub endorsement_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = endorsement_history)
)]
pub struct NewEndorsementHistory {
    pub endorsement_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting endorsements list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndorsementOrderBy {
    pub field: EndorsementField,
    pub direction: Direction,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::EndorsementPolicy;
#[cfg(test)]
mod tests;
