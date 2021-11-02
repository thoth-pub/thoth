use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::graphql::utils::Direction;
use crate::model::Doi;
use crate::model::Ror;
use crate::model::Timestamp;
#[cfg(feature = "backend")]
use crate::schema::institution;
#[cfg(feature = "backend")]
use crate::schema::institution_history;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting institutions list")
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstitutionField {
    #[strum(serialize = "ID")]
    InstitutionId,
    #[strum(serialize = "Institution")]
    InstitutionName,
    #[strum(serialize = "DOI")]
    InstitutionDoi,
    #[strum(serialize = "ROR ID")]
    Ror,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub ror: Option<Ror>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "institution"
)]
pub struct NewInstitution {
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "institution"
)]
pub struct PatchInstitution {
    pub institution_id: Uuid,
    pub institution_name: String,
    pub institution_doi: Option<Doi>,
    pub ror: Option<Ror>,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct InstitutionHistory {
    pub institution_history_id: Uuid,
    pub institution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}

#[cfg_attr(
    feature = "backend",
    derive(Insertable),
    table_name = "institution_history"
)]
pub struct NewInstitutionHistory {
    pub institution_id: Uuid,
    pub account_id: Uuid,
    pub data: serde_json::Value,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Field and order to use when sorting institutions list")
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct InstitutionOrderBy {
    pub field: InstitutionField,
    pub direction: Direction,
}

impl Default for InstitutionField {
    fn default() -> Self {
        InstitutionField::InstitutionName
    }
}

impl fmt::Display for Institution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(doi) = &self.institution_doi {
            write!(f, "{} - {}", &self.institution_name, doi)
        } else {
            write!(f, "{}", &self.institution_name)
        }
    }
}

#[test]
fn test_institutionfield_default() {
    let fundfield: InstitutionField = Default::default();
    assert_eq!(fundfield, InstitutionField::InstitutionName);
}

#[test]
fn test_institutionfield_display() {
    assert_eq!(format!("{}", InstitutionField::InstitutionId), "ID");
    assert_eq!(
        format!("{}", InstitutionField::InstitutionName),
        "Institution"
    );
    assert_eq!(format!("{}", InstitutionField::InstitutionDoi), "DOI");
    assert_eq!(format!("{}", InstitutionField::CreatedAt), "CreatedAt");
    assert_eq!(format!("{}", InstitutionField::UpdatedAt), "UpdatedAt");
}

#[test]
fn test_institutionfield_fromstr() {
    use std::str::FromStr;
    assert_eq!(
        InstitutionField::from_str("ID").unwrap(),
        InstitutionField::InstitutionId
    );
    assert_eq!(
        InstitutionField::from_str("Institution").unwrap(),
        InstitutionField::InstitutionName
    );
    assert_eq!(
        InstitutionField::from_str("DOI").unwrap(),
        InstitutionField::InstitutionDoi
    );
    assert_eq!(
        InstitutionField::from_str("CreatedAt").unwrap(),
        InstitutionField::CreatedAt
    );
    assert_eq!(
        InstitutionField::from_str("UpdatedAt").unwrap(),
        InstitutionField::UpdatedAt
    );
    assert!(InstitutionField::from_str("InstitutionID").is_err());
    assert!(InstitutionField::from_str("Website").is_err());
    assert!(InstitutionField::from_str("Fundings").is_err());
}

#[cfg(feature = "backend")]
pub mod crud;
