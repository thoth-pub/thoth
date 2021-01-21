use chrono::naive::NaiveDateTime;
use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::contributor;

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Field to use when sorting contributors list")
)]
pub enum ContributorField {
    ContributorID,
    FirstName,
    LastName,
    FullName,
    ORCID,
    Website,
    CreatedAt,
    UpdatedAt,
}

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Contributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "contributor"
)]
pub struct NewContributor {
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null = "true"),
    table_name = "contributor"
)]
pub struct PatchContributor {
    pub contributor_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: String,
    pub full_name: String,
    pub orcid: Option<String>,
    pub website: Option<String>,
}
