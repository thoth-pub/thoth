use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::schema::work;
use crate::enumerations::*;

#[derive(Queryable)]
pub struct Work {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub publisher_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "work"]
pub struct NewWork {
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub publisher_id: Uuid,
    pub doi: Option<String>,
    pub publication_date: Option<NaiveDate>,
}

#[derive(Queryable)]
pub struct Publication {
    pub publication_id: Uuid,
    pub publication_type: PublicationType,
    pub work_id: Uuid,
    pub isbn: Option<String>,
    pub publication_url: Option<String>,
}

#[derive(Queryable)]
pub struct Publisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

