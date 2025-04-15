use gloo_storage::{LocalStorage, Storage};
use reqwest::{Body, Client, Method};
use serde::Deserialize;
use serde::Serialize;
use std::future::Future;
use std::str::FromStr;
use thiserror::Error;
use thoth_api::account::model::AccountDetails;
use thoth_api::account::model::LoginCredentials;

use crate::string::STORAGE_ERROR;
use crate::SESSION_KEY;

type HttpFuture = Result<reqwest::Response, reqwest::Error>;

#[derive(Debug, Error, Serialize)]
pub enum AccountError {
    #[error("Authentication error")]
    AuthenticationError,
    #[error("Response error")]
    ResponseError,
}

const HTTP_UNAUTHORIZED: u16 = 401;
const HTTP_FORBIDDEN: u16 = 403;

#[derive(Clone)]
pub struct AccountService {
    http_client: Client,
}

impl AccountService {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    pub fn get_token(&self) -> Option<String> {
        LocalStorage::get(SESSION_KEY).ok()
    }

    pub fn set_token(&self, token: String) {
        self.update_storage(Some(token))
    }

    fn update_storage(&self, token: Option<String>) {
        if let Some(t) = token {
            LocalStorage::set(SESSION_KEY, t).expect(STORAGE_ERROR);
        } else {
            LocalStorage::delete(SESSION_KEY);
        }
    }

    pub fn is_loggedin(&self) -> bool {
        self.get_token().is_some()
    }

    pub fn logout(&self) {
        self.update_storage(None)
    }

    pub async fn login(
        &mut self,
        login_credentials: LoginCredentials,
    ) -> Result<AccountDetails, AccountError> {
        self.post_request::<LoginCredentials, AccountDetails>(
            "/account/login".to_string(),
            login_credentials,
        )
        .await
    }

    pub async fn renew_token(&mut self) -> Result<AccountDetails, AccountError> {
        self.bodyless_post_request::<AccountDetails>("/account/token/renew".to_string())
            .await
    }

    pub async fn account_details(&mut self) -> Result<AccountDetails, AccountError> {
        self.get_request::<AccountDetails>("/account".to_string())
            .await
    }

    async fn request_builder<B, T>(
        &mut self,
        method: &str,
        url: String,
        body: Option<B>,
    ) -> Result<T, AccountError>
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Into<Body> + std::fmt::Debug,
    {
        let res = self.send_request(method, url, body).await.await;
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    let data: Result<T, _> = response.json().await;
                    if let Ok(data) = data {
                        Ok(data)
                    } else {
                        Err(AccountError::ResponseError)
                    }
                } else {
                    match response.status().as_u16() {
                        HTTP_UNAUTHORIZED => Err(AccountError::AuthenticationError),
                        HTTP_FORBIDDEN => Err(AccountError::AuthenticationError),
                        _ => Err(AccountError::ResponseError),
                    }
                }
            }
            Err(_) => Err(AccountError::ResponseError),
        }
    }

    async fn send_request<B>(
        &mut self,
        method: &str,
        url: String,
        body: Option<B>,
    ) -> impl Future<Output = HttpFuture>
    where
        B: Into<Body> + std::fmt::Debug,
    {
        let uri = format!("{}{}", crate::THOTH_GRAPHQL_API, url);
        let verb = Method::from_str(method).unwrap();
        let mut request = self
            .http_client
            .request(verb, uri)
            .header("Content-Type", "application/json");
        if let Some(token) = self.get_token() {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        if let Some(content) = body {
            request = request.body(content);
        }
        request.send()
    }

    async fn get_request<T>(&mut self, url: String) -> Result<T, AccountError>
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.request_builder("GET", url, None::<Body>).await
    }

    async fn bodyless_post_request<T>(&mut self, url: String) -> Result<T, AccountError>
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.request_builder("POST", url, None::<Body>).await
    }

    async fn post_request<B, T>(&mut self, url: String, body: B) -> Result<T, AccountError>
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Serialize,
    {
        let body = serde_json::to_string(&body).expect("Failed to serialise request body");
        self.request_builder("POST", url, Some(body)).await
    }
}
