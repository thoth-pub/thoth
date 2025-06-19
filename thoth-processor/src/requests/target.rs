use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Payload {
    event_type: String
}

pub async fn fire_webhook() -> Result<String, Error> {
    let client = reqwest::Client::new();
    // Will trigger any GitHub Action in this repo with a `repository_dispatch` option set
    // (as long as the "event_type" matches it)
    let url = "https://api.github.com/repos/thoth-pub/thoth-dissemination/dispatches".to_string();
    // For GitHub Actions this can be a "fine-grained access token":
    // https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-dispatch-event--fine-grained-access-tokens
    let token = "placeholder";

    let response = client
        .post(&url)
        .bearer_auth(token)
        // GitHub Actions repository dispatch events require a User-Agent header
        .header("User-Agent", "Thoth")
        // GitHub Actions repository dispatch events require a payload containing "event_type"
        // (this can then be used to control which events trigger which Actions)
        .json(&Payload { event_type: "test".to_string() })
        .send()
        .await?
        .error_for_status()?;

    tracing::info!("response: {:?}", response);
    tracing::info!("response_status: {:?}", response.status());

    Ok(response.status().to_string())
}
