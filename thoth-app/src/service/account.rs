use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use thoth_api::account::model::AccountDetails;
use thoth_api::account::model::LoginCredentials;
use yew::callback::Callback;
use yew::format::Json;
use yew::format::Nothing;
use yew::format::Text;
use yew::services::fetch::FetchService;
use yew::services::fetch::FetchTask;
use yew::services::fetch::Request;
use yew::services::fetch::Response;
use yew::services::storage::Area;
use yew::services::storage::StorageService;

use crate::string::STORAGE_ERROR;
use crate::SESSION_KEY;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Authentication error")]
    AuthenticationError,
    #[error("Response error")]
    ResponseError,
}

const HTTP_UNAUTHORIZED: u16 = 401;
const HTTP_FORBIDDEN: u16 = 403;

pub struct AccountService {}

impl AccountService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_token(&self) -> Option<String> {
        let storage_service = StorageService::new(Area::Local).expect(STORAGE_ERROR);
        if let Ok(token) = storage_service.restore(SESSION_KEY) {
            Some(token)
        } else {
            None
        }
    }

    pub fn set_token(&self, token: String) {
        self.update_storage(Some(token))
    }

    fn update_storage(&self, token: Option<String>) {
        let mut storage_service = StorageService::new(Area::Local).expect(STORAGE_ERROR);
        if let Some(t) = token {
            storage_service.store(SESSION_KEY, Ok(t));
        } else {
            storage_service.remove(SESSION_KEY);
        }
    }

    pub fn is_loggedin(&self) -> bool {
        self.get_token().is_some()
    }

    pub fn logout(&self) {
        self.update_storage(None)
    }

    pub fn login(
        &mut self,
        login_credentials: LoginCredentials,
        callback: Callback<Result<AccountDetails, AccountError>>,
    ) -> FetchTask {
        self.post_request::<LoginCredentials, AccountDetails>(
            "/account/login".to_string(),
            login_credentials,
            callback,
        )
    }

    pub fn renew_token(
        &mut self,
        callback: Callback<Result<AccountDetails, AccountError>>,
    ) -> FetchTask {
        self.bodyless_post_request::<AccountDetails>("/account/token/renew".to_string(), callback)
    }

    pub fn account_details(
        &mut self,
        callback: Callback<Result<AccountDetails, AccountError>>,
    ) -> FetchTask {
        self.get_request::<AccountDetails>("/account".to_string(), callback)
    }

    pub fn check_version(
        &mut self,
        callback: Callback<Result<String, AccountError>>,
    ) -> FetchTask {
        self.bodyless_post_request::<String>("/version".to_string(), callback)
    }

    fn request_builder<B, T>(
        &mut self,
        method: &str,
        url: String,
        body: B,
        callback: Callback<Result<T, AccountError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Into<Text> + std::fmt::Debug,
    {
        let handler = move |response: Response<Text>| {
            if let (meta, Ok(data)) = response.into_parts() {
                if meta.status.is_success() {
                    let data: Result<T, _> = serde_json::from_str(&data);
                    if let Ok(data) = data {
                        callback.emit(Ok(data))
                    } else {
                        callback.emit(Err(AccountError::ResponseError))
                    }
                } else {
                    match meta.status.as_u16() {
                        HTTP_UNAUTHORIZED => callback.emit(Err(AccountError::AuthenticationError)),
                        HTTP_FORBIDDEN => callback.emit(Err(AccountError::AuthenticationError)),
                        _ => callback.emit(Err(AccountError::ResponseError)),
                    }
                }
            } else {
                callback.emit(Err(AccountError::ResponseError))
            }
        };

        let url = format!("{}{}", crate::THOTH_API, url);
        let mut builder = Request::builder()
            .method(method)
            .uri(url.as_str())
            .header("Content-Type", "application/json");
        if let Some(token) = self.get_token() {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        let request = builder.body(body).unwrap();

        FetchService::fetch(request, handler.into()).unwrap()
    }

    fn get_request<T>(
        &mut self,
        url: String,
        callback: Callback<Result<T, AccountError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.request_builder("GET", url, Nothing, callback)
    }

    fn bodyless_post_request<T>(
        &mut self,
        url: String,
        callback: Callback<Result<T, AccountError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.request_builder("POST", url, Nothing, callback)
    }

    fn post_request<B, T>(
        &mut self,
        url: String,
        body: B,
        callback: Callback<Result<T, AccountError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Serialize,
    {
        let body: Text = Json(&body).into();
        self.request_builder("POST", url, body, callback)
    }
}
