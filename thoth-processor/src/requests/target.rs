use loco_rs::prelude::*;
use uuid::Uuid;

pub async fn fire_webhook(
    url: String,
    token: Option<String>,
    work_id: Uuid,
    payload: Option<String>,
) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let mut request = client
        .post(&url)
        // GitHub Actions repository dispatch events require a User-Agent header
        .header("User-Agent", "Thoth");

    if let Some(token_value) = token {
        request = request.bearer_auth(token_value);
    }

    // References for constructing payloads:
    // GitHub Actions: https://docs.github.com/en/actions/reference/events-that-trigger-workflows#repository_dispatch
    // Mattermost: https://developers.mattermost.com/integrate/webhooks/incoming/
    if let Some(payload_value) = payload {
        let interpolated_payload = payload_value.replace("${work_id}", &work_id.to_string());
        request = request.body(interpolated_payload);
    }

    let response = request
        .send()
        .await?
        .error_for_status()?;

    tracing::info!("response: {:?}", response);
    tracing::info!("response_status: {:?}", response.status());

    Ok(response.status().to_string())
}
