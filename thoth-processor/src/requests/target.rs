use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
pub struct ClientPayload {
    work_id: Uuid,
    platform: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Payload {
    event_type: String,
    client_payload: ClientPayload,
}

pub async fn fire_webhook(
    url: String,
    token: Option<String>,
    work_id: Uuid,
    platform: Option<String>,
) -> Result<String, Error> {
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
            client_payload: ClientPayload {
                work_id: work_id,
                platform: platform.unwrap_or_default(),
            },
        })
        .send()
        .await?
        .error_for_status()?;

    tracing::info!("response: {:?}", response);
    tracing::info!("response_status: {:?}", response.status());

    Ok(response.status().to_string())
}
