#[allow(clippy::upper_case_acronyms)]
mod queries;

use graphql_client::GraphQLQuery;
use graphql_client::Response;
use serde::Serialize;
use std::future::Future;
use thoth_api::errors::{ThothError, ThothResult};
use uuid::Uuid;

pub use crate::queries::work_query::*;
use crate::queries::{work_query, works_query, WorkQuery, WorksQuery};

type HttpFuture = Result<reqwest::Response, reqwest::Error>;

pub struct ThothClient {
    graphql_endpoint: String,
    http_client: reqwest::Client,
}

impl ThothClient {
    pub fn new(graphql_endpoint: String) -> Self {
        ThothClient {
            graphql_endpoint,
            http_client: reqwest::Client::new(),
        }
    }

    async fn post_request<T: Serialize + ?Sized>(
        self,
        request_body: &T,
    ) -> impl Future<Output = HttpFuture> {
        self.http_client
            .post(&self.graphql_endpoint)
            .json(&request_body)
            .send()
    }

    pub async fn get_work(self, work_id: Uuid) -> ThothResult<Work> {
        let request_body = WorkQuery::build_query(work_query::Variables { work_id });
        let res = self.post_request(&request_body).await.await?;
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

    pub async fn get_works(self, publishers: Option<Vec<Uuid>>) -> ThothResult<Vec<Work>> {
        let request_body = WorksQuery::build_query(works_query::Variables { publishers });
        let res = self.post_request(&request_body).await.await?;
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
}
