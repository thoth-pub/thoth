use std::fmt;

use chrono::naive::NaiveDate;
use graphql_client::GraphQLQuery;
use thoth_api::model::Doi;
use thoth_api::model::Isbn;
use thoth_api::model::Orcid;
use thoth_api::model::Ror;
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorkQuery;

impl fmt::Display for work_query::LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for work_query::SubjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorksQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize,PartialEq",
    variables_derives = "Debug,PartialEq"
)]
pub struct WorkCountQuery;

// Needed to set work_query::Work as the canonical struct for the shared fragment in the two queries
// until https://github.com/graphql-rust/graphql-client/issues/312 gets fixed
impl From<works_query::Work> for work_query::Work {
    fn from(w: works_query::Work) -> Self {
        let se = serde_json::to_string(&w).unwrap();
        serde_json::from_str(&se).unwrap()
    }
}

// As above: enables shared processing of parent Works and child RelatedWorks in doideposit format
impl From<work_query::Work> for work_query::WorkRelationsRelatedWork {
    fn from(w: work_query::Work) -> Self {
        let se = serde_json::to_string(&w).unwrap();
        serde_json::from_str(&se).unwrap()
    }
}
