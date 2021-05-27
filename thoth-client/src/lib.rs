#[allow(clippy::upper_case_acronyms)]
mod queries;

use graphql_client::GraphQLQuery;
use graphql_client::Response;
use reqwest::IntoUrl;
use serde::Serialize;
use std::future::Future;
use thoth_api::errors::{ThothError, ThothResult};
use uuid::Uuid;

pub use crate::queries::work_query::*;
use crate::queries::{work_query, works_query, WorkQuery, WorksQuery};

type HttpFuture = Result<reqwest::Response, reqwest::Error>;

/// A GraphQL `ThothClient` to query metadata
pub struct ThothClient<U: IntoUrl> {
    graphql_endpoint: U,
    http_client: reqwest::Client,
}

impl<U> ThothClient<U>
where
    U: IntoUrl,
{
    /// Constructs a new `ThothClient`
    pub fn new(graphql_endpoint: U) -> Self {
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
            .post(self.graphql_endpoint)
            .json(&request_body)
            .send()
    }

    /// Get a `Work` from Thoth given its `work_id`
    ///
    /// # Errors
    ///
    /// This method fails if the `work_id` was not found
    /// or if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_api::errors::ThothResult;
    /// # use thoth_client::{ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Work> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql");
    /// let work_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work = thoth_client.get_work(work_id).await?;
    /// # Ok(work)
    /// # }
    /// ```
    pub async fn get_work(self, work_id: Uuid) -> ThothResult<Work> {
        let request_body = WorkQuery::build_query(work_query::Variables { work_id });
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<work_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.work.work),
            None => Err(ThothError::EntityNotFound),
        }
    }

    /// Get a list of `Work`s from Thoth
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending the request
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use thoth_api::errors::ThothResult;
    /// # use thoth_client::{ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Vec<Work>> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql");
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let works = thoth_client.get_works(Some(vec![publisher_id])).await?;
    /// # Ok(works)
    /// # }
    /// ```
    pub async fn get_works(self, publishers: Option<Vec<Uuid>>) -> ThothResult<Vec<Work>> {
        let request_body = WorksQuery::build_query(works_query::Variables { publishers });
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<works_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.works.iter().map(|w| w.work.clone().into()).collect()), // convert works_query::Work into work_query::Work
            None => Err(ThothError::EntityNotFound),
        }
    }
}
