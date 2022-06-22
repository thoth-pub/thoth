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
    reqwest::Client::new()
        // TODO remove this hard-coding (relative URLs without base are not permitted)
        .get("http://localhost:8080/manifest.json")
        .header("Content-Type", "application/json")
        .send()
}
