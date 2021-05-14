use std::fmt;

use chrono::naive::NaiveDate;
use graphql_client::{GraphQLQuery, Response};
use thoth_api::errors::{ThothError, ThothResult};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "assets/schema.json",
    query_path = "assets/work_query.graphql",
    response_derives = "Debug"
)]
pub struct WorkQuery;

pub async fn get_work(work_id: Uuid, gql_endpoint: &str) -> ThothResult<work_query::WorkQueryWork> {
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
            Ok(data.work)
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
    query_path = "assets/works_query.graphql",
    response_derives = "Debug"
)]
pub struct WorksQuery;

pub async fn get_works(gql_endpoint: &str) -> ThothResult<Vec<works_query::WorksQueryWorks>> {
    let request_body = WorksQuery::build_query(works_query::Variables);
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
            Ok(data.works)
        }
        _ => Err(ThothError::InternalError("Query failed".to_string())),
    }
}
