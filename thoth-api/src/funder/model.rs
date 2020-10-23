use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::funder;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Funder {
    pub funder_id: Uuid,
    pub funder_name: String,
    pub funder_doi: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "funder")]
pub struct NewFunder {
    pub funder_name: String,
    pub funder_doi: Option<String>,
}
