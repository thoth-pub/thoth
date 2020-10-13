use uuid::Uuid;

#[cfg(feature = "backend")]
use crate::schema::imprint;
#[cfg(feature = "backend")]
use crate::schema::publisher;

#[cfg_attr(feature = "backend", derive(Queryable))]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[cfg_attr(feature = "backend", derive(juniper::GraphQLInputObject, Insertable))]
#[cfg_attr(feature = "backend", table_name = "publisher")]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

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
