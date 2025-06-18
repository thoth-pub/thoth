use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use thoth_api::event::model::Event;
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksVariables {
    work_id: Uuid,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksQueryBody {
    pub query: String,
    pub variables: WebhooksVariables,
}

pub async fn query_webhooks(event: Event) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let url = "https://api.thoth.pub/graphql".to_string();
    let query =
        "
query WebhooksQuery($workId: Uuid!) {
    work(workId: $workId) {
        workId
        fullTitle
    }
}".to_string();

    let variables = WebhooksVariables {
        work_id: event.work_id,
    };
    let body = WebhooksQueryBody {
        query,
        variables,
    };
    let token = "placeholder".to_string();

    let response = client
        .post(&url)
        .json(&body)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    tracing::info!("response: {:?}", response);
    let response_text = response.text().await?;
    tracing::info!("response: {:?}", response_text);

    Ok(response_text)
}
