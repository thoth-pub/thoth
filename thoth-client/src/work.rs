use std::fmt;

use chrono::naive::NaiveDate;
use graphql_client::{GraphQLQuery, Response};
use thoth_api::errors::{ThothError, ThothResult};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Deserialize,Serialize"
)]
pub struct WorkQuery;

pub async fn get_work(work_id: Uuid, gql_endpoint: &str) -> ThothResult<work_query::Work> {
    let request_body = WorkQuery::build_query(work_query::Variables { work_id });
    let client = reqwest::Client::new();
    let res = client.post(gql_endpoint).json(&request_body).send().await?;
    let response_body: Response<work_query::ResponseData> = res.json().await?;
    match response_body.data {
        Some(data) => {
            if let Some(errors) = response_body.errors {
                println!("there are errors:");
                for error in &errors {
                    println!("{:?}", error);
                }
            }
            Ok(data.work.work)
        }
        None => Err(ThothError::EntityNotFound),
    }
}

impl fmt::Display for work_query::LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/queries.graphql",
    response_derives = "Debug,Clone,Deserialize,Serialize"
)]
pub struct WorksQuery;

pub async fn get_works(
    publishers: Option<Vec<Uuid>>,
    gql_endpoint: &str,
) -> ThothResult<Vec<work_query::Work>> {
    let request_body = WorksQuery::build_query(works_query::Variables { publishers });
    let client = reqwest::Client::new();
    let res = client.post(gql_endpoint).json(&request_body).send().await?;
    let response_body: Response<works_query::ResponseData> = res.json().await?;
    match response_body.data {
        Some(data) => {
            if let Some(errors) = response_body.errors {
                println!("there are errors:");
                for error in &errors {
                    println!("{:?}", error);
                }
            }
            Ok(data.works.iter().map(|w| w.work.clone().into()).collect())
        }
        _ => Err(ThothError::InternalError("Query failed".to_string())),
    }
}

// Needed to set work_query::Work as the canonical struct for the shared fragment in the two queries
// until https://github.com/graphql-rust/graphql-client/issues/312 gets fixed
impl From<works_query::Work> for work_query::Work {
    fn from(w: works_query::Work) -> Self {
        let se = serde_json::to_string(&w).unwrap();
        serde_json::from_str(&se).unwrap()
    }
}
