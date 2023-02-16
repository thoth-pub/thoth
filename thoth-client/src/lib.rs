mod parameters;
// GraphQLQuery derive macro breaks this linting rule - ignore while awaiting fix
#[allow(clippy::derive_partial_eq_without_eq)]
mod queries;

use graphql_client::GraphQLQuery;
use graphql_client::Response;
use serde::Serialize;
use std::future::Future;
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

pub use crate::parameters::QueryParameters;
use crate::parameters::{WorkQueryVariables, WorksQueryVariables};
pub use crate::queries::work_query::*;
use crate::queries::{work_query, works_query, WorkQuery, WorksQuery};

type HttpFuture = Result<reqwest::Response, reqwest::Error>;

/// A GraphQL `ThothClient` to query metadata
pub struct ThothClient {
    graphql_endpoint: String,
    http_client: reqwest::Client,
}

impl ThothClient {
    /// Constructs a new `ThothClient`
    pub fn new(graphql_endpoint: String) -> Self {
        ThothClient {
            graphql_endpoint,
            http_client: reqwest::Client::new(),
        }
    }

    async fn post_request<T: Serialize + ?Sized>(
        &self,
        request_body: &T,
    ) -> impl Future<Output = HttpFuture> {
        self.http_client
            .post(&self.graphql_endpoint)
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
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{QueryParameters, ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Work> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let work_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let work = thoth_client.get_work(work_id, QueryParameters::new()).await?;
    /// # Ok(work)
    /// # }
    /// ```
    pub async fn get_work(&self, work_id: Uuid, parameters: QueryParameters) -> ThothResult<Work> {
        let variables: work_query::Variables = WorkQueryVariables::new(work_id, parameters).into();
        let request_body = WorkQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<work_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.work),
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
    /// # use thoth_errors::ThothResult;
    /// # use thoth_client::{QueryParameters, ThothClient, Work};
    /// # use uuid::Uuid;
    ///
    /// # async fn run() -> ThothResult<Vec<Work>> {
    /// let thoth_client = ThothClient::new("https://api.thoth.pub/graphql".to_string());
    /// let publisher_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?;
    /// let works = thoth_client.get_works(Some(vec![publisher_id]), 100, 0, QueryParameters::new()).await?;
    /// # Ok(works)
    /// # }
    /// ```
    pub async fn get_works(
        &self,
        publishers: Option<Vec<Uuid>>,
        limit: i64,
        offset: i64,
        parameters: QueryParameters,
    ) -> ThothResult<Vec<Work>> {
        let variables: works_query::Variables =
            WorksQueryVariables::new(publishers, limit, offset, parameters).into();
        let request_body = WorksQuery::build_query(variables);
        let res = self.post_request(&request_body).await.await?;
        let response_body: Response<works_query::ResponseData> = res.json().await?;
        match response_body.data {
            Some(data) => Ok(data.works.iter().map(|w| w.clone().into()).collect()), // convert works_query::Work into work_query::Work
            None => Err(ThothError::EntityNotFound),
        }
    }
}
