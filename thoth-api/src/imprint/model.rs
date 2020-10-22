use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::imprint;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Imprint {
    pub imprint_id: Uuid,
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "imprint")]
pub struct NewImprint {
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}
