use semver::Version;
use serde_json::Value;
use std::future::Future;
use thoth_errors::ThothError;

type HttpFuture = Result<reqwest::Response, reqwest::Error>;

pub async fn get_version() -> Result<Version, ThothError> {
    let response = get_request().await.await?;
    if response.status().is_success() {
        let response_body: Result<Value, _> = response.json().await;
        match response_body {
            Ok(data) => match Version::parse(data["version"].as_str().unwrap_or_default()) {
                Ok(version) => Ok(version),
                Err(_) => Err(ThothError::InternalError(
                    "No valid version information found".to_string(),
                )),
            },
            Err(e) => Err(ThothError::InternalError(e.to_string())),
        }
    } else {
        Err(ThothError::InternalError(response.status().to_string()))
    }
}

async fn get_request() -> impl Future<Output = HttpFuture> {
    let base_url = web_sys::window().unwrap().origin();
    reqwest::Client::new()
        .get(format!("{base_url}/admin/manifest.json"))
        .header("Content-Type", "application/json")
        .send()
}
