use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Payload {
    event_type: String,
}

pub async fn fire_webhook(url: String, token: Option<String>) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .bearer_auth(token.unwrap_or_default())
        // GitHub Actions repository dispatch events require a User-Agent header
        .header("User-Agent", "Thoth")
        // GitHub Actions repository dispatch events require a payload containing "event_type"
        // (this can then be used to control which events trigger which Actions)
        // (it also seems to determine the name given to any ensuing workflow runs)
        .json(&Payload {
            event_type: "test".to_string(),
        })
        .send()
        .await?
        .error_for_status()?;

    tracing::info!("response: {:?}", response);
    tracing::info!("response_status: {:?}", response.status());

    Ok(response.status().to_string())
}
