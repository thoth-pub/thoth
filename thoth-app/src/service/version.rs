use thoth_api::errors::ThothError;
use yew::callback::Callback;
use yew::format::Nothing;
use yew::format::Text;
use yew::services::fetch::FetchService;
use yew::services::fetch::FetchTask;
use yew::services::fetch::Request;
use yew::services::fetch::Response;

use crate::THOTH_API;

pub fn check_version(callback: Callback<Result<String, ThothError>>) -> FetchTask {
    let handler = move |response: Response<Text>| {
        if let (meta, Ok(data)) = response.into_parts() {
            if meta.status.is_success() {
                let data: Result<String, _> = serde_json::from_str(&data);
                match data {
                    Ok(data) => callback.emit(Ok(data)),
                    Err(e) => callback.emit(Err(ThothError::InternalError(e.to_string()))),
                }
            } else {
                callback.emit(Err(ThothError::InternalError(meta.status.to_string())))
            }
        } else {
            callback.emit(Err(ThothError::InternalError(
                "Could not parse HTTP response".to_string(),
            )))
        }
    };

    let builder = Request::builder()
        .method("POST")
        .uri(format!("{}{}", THOTH_API, "/version".to_string()))
        .header("Content-Type", "application/json");
    let request = builder.body(Nothing).unwrap();

    FetchService::fetch(request, handler.into()).unwrap()
}
