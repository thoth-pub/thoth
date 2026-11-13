use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::graphql::types::inputs::Direction;
use crate::model::{CountryCode, Doi, Ror, Timestamp};
#[cfg(feature = "backend")]
use crate::schema::institution;
#[cfg(feature = "backend")]
use crate::schema::institution_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionField {
    #[strum(serialize = "ID")]
    InstitutionId,
    #[strum(serialize = "Institution")]
    #[default]
    InstitutionName,
    #[strum(serialize = "DOI")]
    InstitutionDoi,
    #[strum(serialize = "ROR ID")]
    Ror,
    #[strum(serialize = "Country")]
    CountryCode,
    CreatedAt,
    UpdatedAt,
    #[strum(serialize = "ROR Status")]
    RorStatus,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(
        description = "Status of an organisation's record in the Research Organization Registry (ROR)"
    ),
    ExistingTypePath = "crate::schema::sql_types::RorStatus"
)]
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize, Serialize, EnumString, Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "title_case")]
pub enum RorStatus {
    #[cfg_attr(feature = "backend", graphql(description = "The ROR record is active"))]
    #[default]
    Active,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The ROR record is inactive: the organisation no longer operates, or has been succeeded by another"
        )
    )]
    Inactive,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The ROR record has been withdrawn, for example because it duplicated another record"
        )
    )]
    Withdrawn,
}

#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
    // Snapshots serialized before ROR-SYNC-01C lack these fields;
    // deserialization must not require them, so the column defaults apply.
    #[serde(default)]
    pub ror_status: RorStatus,
    #[serde(default)]
    pub ror_successor: Option<Ror>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, diesel::Insertable),
    graphql(description = "Set of values required to define a new organisation with which contributors may be affiliated or by which works may be funded"),
    diesel(table_name = institution)
)]
pub struct NewInstitution {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

/// The GraphQL input for `updateInstitution`.
///
/// It is not a Diesel changeset. The model's update writes it through two
/// changesets in one `UPDATE`: [`InstitutionChangeset`] for the fields every
/// authenticated caller may write, and [`InstitutionRorStatePatch`] for the ROR
/// status and successor, which only `SUPERUSER` or `ROR_SYNC_SERVICE` may set.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Set of values required to update an existing organisation with which contributors may be affiliated or by which works may be funded"
    )
)]
pub struct PatchInstitution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
    /// Absent means unchanged.
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Status of the ROR record named by `ror`. Omit unless the caller holds `ROR_SYNC_SERVICE` or `SUPERUSER`. A caller without either that sends a value differing from the stored one is denied the whole update; some clients, such as `graphql_client`, send an unset optional field as an explicit null, which is such a value when a successor is stored."
        )
    )]
    pub ror_status: Option<RorStatus>,
    /// Absent means unchanged, an explicit `null` clears it, and a value sets it.
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "ROR ID of the single successor that ROR lists for an inactive or withdrawn record. Omit unless the caller holds `ROR_SYNC_SERVICE` or `SUPERUSER`. A caller without either that sends a value differing from the stored one is denied the whole update; some clients, such as `graphql_client`, send an unset optional field as an explicit null, which is such a value when a successor is stored."
        )
    )]
    pub ror_successor: juniper::Nullable<Ror>,
}

/// The changeset for the institution fields that any authenticated caller may
/// write. A `None` writes NULL, as `PatchInstitution` always did.
#[cfg_attr(
    feature = "backend",
    derive(diesel::AsChangeset),
    diesel(table_name = institution, treat_none_as_null = true)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstitutionChangeset {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
    pub country_code: Option<CountryCode>,
}

impl From<&PatchInstitution> for InstitutionChangeset {
    fn from(patch: &PatchInstitution) -> Self {
        Self {
            institution_name: patch.institution_name.clone(),
            institution_doi: patch.institution_doi.clone(),
            ror: patch.ror.clone(),
            country_code: patch.country_code,
        }
    }
}

/// The changeset for `ror_status` and `ror_successor`.
///
/// An outer `None` leaves the column out of the `UPDATE`; `Some(None)` writes
/// NULL to `ror_successor`. The model's update builds it from the caller's
/// privilege and from whether `ror` changes; an unprivileged caller's own
/// values are never copied into it.
#[cfg_attr(
    feature = "backend",
    derive(diesel::AsChangeset),
    diesel(table_name = institution)
)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstitutionRorStatePatch {
    pub ror_status: Option<RorStatus>,
    pub ror_successor: Option<Option<Ror>>,
}

/// A row of `institution_history`. After ROR-SYNC-01C (thoth#1024) a row holds
/// one of:
///
/// - for a create through `createInstitution`, the created row;
/// - for an update, the row as it was before the update;
/// - a merge record written by ROR-SYNC-01B's migration (`user_id`
///   `ROR-SYNC-01B`, `data` a merge object) or, under its forward repair, by
///   the repair script (`user_id` `ROR-SYNC-01B-REPAIR`);
/// - a row moved from an institution merged into this one, whose `data` names
///   the merged institution's ID.
///
/// For a create or an update, `data` is a JSON string of the serialised
/// `Institution` and `user_id` is the caller's user ID.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
pub struct InstitutionHistory {
    pub institution_history_id: Uuid,
    pub institution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(diesel::Insertable),
    diesel(table_name = institution_history)
)]
pub struct NewInstitutionHistory {
    pub institution_id: Uuid,
    pub user_id: String,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstitutionOrderBy {
    pub field: InstitutionField,
    pub direction: Direction,
}

impl fmt::Display for Institution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ror) = &self.ror {
            write!(f, "{} - {}", self.institution_name, ror)
        } else if let Some(doi) = &self.institution_doi {
            write!(f, "{} - {}", self.institution_name, doi)
        } else {
            write!(f, "{}", self.institution_name)
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(feature = "backend")]
mod policy;
#[cfg(feature = "backend")]
pub(crate) use policy::InstitutionPolicy;
#[cfg(test)]
mod tests;
