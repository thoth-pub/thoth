use uuid::Uuid;

use crate::schema::imprint;
use crate::schema::publisher;

#[derive(Queryable)]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "publisher"]
pub struct NewPublisher {
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Queryable)]
pub struct Imprint {
    pub imprint_id: Uuid,
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "imprint"]
pub struct NewImprint {
    pub publisher_id: Uuid,
    pub imprint_name: String,
    pub imprint_url: Option<String>,
}
