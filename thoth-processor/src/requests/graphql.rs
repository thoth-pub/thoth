use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use thoth_api::event::model::{Event, EventType};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksVariables {
    work_id: Uuid,
    event_types: Vec<EventType>,
    is_published: bool,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksQueryBody {
    pub query: String,
    pub variables: WebhooksVariables,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub endpoint: String,
    pub token: Option<String>,
    pub is_published: bool,
    pub event_type: EventType,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksResponsePublisher {
    pub webhooks: Vec<Webhook>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksResponseImprint {
    pub publisher: WebhooksResponsePublisher,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksResponseWork {
    pub imprint: WebhooksResponseImprint,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksResponseData {
    pub work: WebhooksResponseWork,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhooksResponseBody {
    pub data: WebhooksResponseData
}

pub async fn query_webhooks(event: Event) -> Result<Vec<Webhook>, Error> {
    let client = reqwest::Client::new();
    let url = "https://api.thoth.pub/graphql".to_string();
    let query =
        "
query WebhooksQuery($workId: Uuid!, $eventTypes: [EventType!], $isPublished: Boolean!) {
    work(workId: $workId) {
        imprint {
            publisher {
                webhooks(eventTypes: $eventTypes, isPublished: $isPublished) {
                    endpoint
                    token
                    isPublished
                    eventType
                }
            }
        }
    }
}".to_string();

    let variables = WebhooksVariables {
        work_id: event.work_id,
        event_types: vec![event.event_type],
        is_published: event.is_published,
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
    let response_text = response.json::<WebhooksResponseBody>().await?;
    tracing::info!("response_text: {:?}", response_text);

    Ok(response_text.data.work.imprint.publisher.webhooks)
}
