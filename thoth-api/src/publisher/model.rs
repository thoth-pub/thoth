use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::publisher;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, Insertable),
    table_name = "publisher",
)]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject, AsChangeset),
    changeset_options(treat_none_as_null="true"),
    table_name = "publisher",
)]
pub struct PatchPublisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}
